use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};

use rand::{Rng, RngCore};

use crate::utils::{Log, Speed};

use super::damages;
use super::ship_internals::*;
use crate::game::ship::damages::DamageToApply;

#[derive(Clone,Debug)]
pub enum CombatLog {
    NoTarget { id: ShipInstanceId },
    Recharging { id: ShipInstanceId, weapon_id: ComponentId, wait_time: f32 },
    Miss { id: ShipInstanceId, target_id: ShipInstanceId, weapon_id: ComponentId},
    Hit { id: ShipInstanceId, target_id: ShipInstanceId, damage: Damage, weapon_id: ComponentId, armor_index: ArmorIndex, hull_damage: bool },
    ComponentDestroy { id: ShipInstanceId, component_id: ComponentId },
    ShipDestroyed { id: ShipInstanceId },
}

/// Short-lived context used to run single combat run
pub struct CombatContext<'a> {
    delta_time: f32,
    total_time: f32,
    ships: HashMap<ShipInstanceId, &'a mut ShipInstance>,
    distances: HashMap<(ShipInstanceId, ShipInstanceId), f32>,
    components: &'a Components,
}

impl<'a> CombatContext<'a> {
    pub fn new(components: &'a Components) -> Self {
        CombatContext {
            delta_time: 0.0,
            total_time: 0.0,
            ships: HashMap::new(),
            distances: HashMap::new(),
            components: components,
        }
    }

    pub fn add_ship(&mut self, ship: &'a mut ShipInstance) {
        if self.ships.contains_key(&ship.id) {
            panic!();
        }
        self.ships.insert(ship.id, ship);
    }

    pub fn get_ships(&self) -> Vec<ShipInstance> {
        let mut vec = vec![];
        for (_, ship) in &self.ships {
            // remove clone
            let instance: ShipInstance = ShipInstance::clone(ship);
            vec.push(instance);
        }

        vec
    }

    pub fn set_distance(&mut self, id0: ShipInstanceId, id1: ShipInstanceId, distance: f32) {
        self.distances.insert((id0, id1), distance);
        self.distances.insert((id1, id0), distance);
    }

    pub fn set_time(&mut self, delta_time: f32, total_time: f32) {
        self.delta_time = delta_time;
        self.total_time = total_time;
    }
}

pub struct Combat {

}

impl Combat {

    pub fn execute(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>) {
        let mut damages = Combat::execute_attacks(ctx, logs);
        damages::apply_damages(ctx.components, logs, &mut ctx.ships, damages);
    }

    fn execute_attacks(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>) -> Vec<DamageToApply> {
        let mut damages = vec![];
        let ship_sequence = Combat::roll_order(&ctx.ships);

        for ship_id in ship_sequence {
            Combat::execute_attack(ctx, logs, &mut damages, ship_id);
        }

        damages
    }

    fn execute_attack(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, damages: &mut Vec<DamageToApply>, attacker_id: ShipInstanceId) {
        let target_id = match Combat::search_best_target(ctx, attacker_id) {
            Some(target_id) => target_id,
            None => {
                logs.push(CombatLog::NoTarget { id: attacker_id });
                return;
            }
        };

        Combat::execute_fire_at(ctx, logs, damages, attacker_id, target_id);
    }

    fn execute_fire_at(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, damages: &mut Vec<DamageToApply>, attacker_id: ShipInstanceId, target_id: ShipInstanceId) {
        let defender_stats = {
            let ship = ctx.ships.get(&target_id).unwrap();
            (ship.current_stats.total_width, ship.current_stats.speed)
        };

        let mut attacker = ctx.ships.get_mut(&attacker_id).unwrap();
        let weapons = attacker.spec.find_weapons(ctx.components);

        for weapon_id in weapons {
            let weapon = ctx.components.get(&weapon_id).weapon.as_ref().unwrap();
            let amount = *attacker.spec.amount(&weapon_id).unwrap();

            let hit_chance = Combat::compute_hit_chance(weapon, attacker.current_stats.speed, defender_stats.0, defender_stats.1);

            for i in 0..amount {
                let weapon_state = attacker.get_weapon_state(&weapon_id, i);

                if weapon_state.recharge > 0.0 {
                    weapon_state.recharge -= ctx.delta_time;
                }

                let can_fire = weapon_state.recharge <= 0.0;
                if can_fire {
                    weapon_state.recharge += weapon.reload;

                    for _ in 0..weapon.rounds {
                        if Combat::roll(hit_chance) {
                            damages.push(DamageToApply {
                                attacker_id,
                                target_id,
                                amount: weapon.damage,
                                weapon_id,
                                damage_type: weapon.damage_type.clone(),
                            });
                        } else {
                            logs.push(CombatLog::Miss {
                                id: attacker_id,
                                target_id: target_id,
                                weapon_id: weapon_id
                            });
                        }
                    }
                } else {
                    logs.push(CombatLog::Recharging { id: attacker_id, weapon_id: weapon_id, wait_time: weapon_state.recharge });
                }
            }
        }
    }

    // TODO: the ration should not be speed speed, but tracking speed vs difference in speed
    /// =POW(0.5, B12/A12)+POW(0.1, 100 /C12)
    fn compute_hit_chance(weapon: &Weapon, attack_speed: Speed, target_width: u32, target_speed: Speed) -> f32 {
        let speed_ration: f32 = 0.5_f32.powf(target_speed.0 / attack_speed.0);
        let size_bonus: f32 = 0.1_f32.powf(100.0 / target_width as f32);
        let value = speed_ration + size_bonus;
        if value < 0.01 || value > 0.99 {
            Log::warn("combat", &format!("hit chance {:?}, target {:?}, width {:?}. speed_ration {:?}, size_bonus {:?}, value {:?}", attack_speed, target_speed, target_width, speed_ration, size_bonus, value));
        } else {
            Log::debug("combat", &format!("hit chance {:?}, target {:?}, width {:?}. speed_ration {:?}, size_bonus {:?}, value {:?}", attack_speed, target_speed, target_width, speed_ration, size_bonus, value));
        }
        value
    }

    fn roll(chance: f32) -> bool {
        let mut rng = rand::thread_rng();
        let value: f32 = rng.gen();
        value <= chance
    }

    fn is_weapon_ready(ctx: &mut CombatContext, attacker_id: ShipInstanceId) -> bool {
        let _ship = ctx.ships.get(&attacker_id).unwrap();
        true
    }

    fn search_best_target(ctx: &mut CombatContext, attacker_id: ShipInstanceId) -> Option<ShipInstanceId> {
        for candidate_id in ctx.ships.keys() {
            if *candidate_id != attacker_id {
                return Some(*candidate_id);
            }
        }

        None
    }

    fn roll_order(ships: &HashMap<ShipInstanceId, &mut ShipInstance>) -> Vec<ShipInstanceId> {
        ships.keys().map(|i| *i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hit_chance_test() {
        fn test(attack_speed: f32, target_speed: f32, target_width: u32, expected: f32) {
            let weapon = Weapon {
                damage: Damage(1),
                reload: 1.0,
                rounds: 1,
                damage_type: WeaponDamageType::Explosive
            };

            let hit_chance = Combat::compute_hit_chance(&weapon, Speed(attack_speed), target_width, Speed(target_speed));
            assert_eq!(hit_chance, expected);
        }

        test(0.5, 0.5, 10, 0.5);
        test(1.0, 2.0, 10, 0.25);
        test(1.0, 10.0, 10, 0.0009765626);
        test(1.0, 10.0, 100, 0.100976564);
    }
}
