use std::collections::HashMap;

use rand::Rng;

use crate::utils::Speed;

use super::damages;
use super::ship_internals::*;
use crate::game::ship::damages::DamageToApply;

#[derive(Clone, Debug)]
pub enum CombatLog {
    NoTarget {
        id: ShipInstanceId,
    },
    Recharging {
        id: ShipInstanceId,
        weapon_id: ComponentId,
        wait_time: f32,
    },
    Miss {
        id: ShipInstanceId,
        target_id: ShipInstanceId,
        weapon_id: ComponentId,
    },
    Hit {
        id: ShipInstanceId,
        target_id: ShipInstanceId,
        damage: Damage,
        weapon_id: ComponentId,
        armor_index: ArmorIndex,
        hull_damage: bool,
    },
    ComponentDestroy {
        id: ShipInstanceId,
        component_id: ComponentId,
    },
    ShipDestroyed {
        id: ShipInstanceId,
    },
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

struct WeaponFire {
    weapons: Vec<ComponentId>,
    target_id: ShipInstanceId,
}

pub struct Combat {}

impl Combat {
    pub fn execute(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>) {
        let targets = Combat::acquire_targets(ctx, logs);
        let fires = Combat::fire_weapons(ctx, logs, targets);
        let damages = Combat::compute_hits(ctx, logs, fires);
        damages::apply_damages(ctx.components, logs, &mut ctx.ships, damages);
    }

    fn acquire_targets(
        ctx: &CombatContext,
        logs: &mut Vec<CombatLog>,
    ) -> HashMap<ShipInstanceId, ShipInstanceId> {
        ctx.ships
            .iter()
            .filter(|(_, ship)| !ship.wreck)
            .flat_map(
                |(attacker_id, _)| match Combat::search_best_target(ctx, *attacker_id) {
                    Some(target_id) => Some((*attacker_id, target_id)),
                    None => {
                        logs.push(CombatLog::NoTarget { id: *attacker_id });
                        None
                    }
                },
            )
            .collect()
    }

    fn fire_weapons(
        ctx: &mut CombatContext,
        logs: &mut Vec<CombatLog>,
        targeting: HashMap<ShipInstanceId, ShipInstanceId>,
    ) -> HashMap<ShipInstanceId, WeaponFire> {
        let mut result: HashMap<ShipInstanceId, WeaponFire> = HashMap::new();

        for (attacker_id, attacker) in ctx.ships.iter_mut() {
            let weapons = attacker.spec.find_weapons(ctx.components);
            for weapon_id in weapons {
                let weapon = ctx.components.get(&weapon_id).weapon.as_ref().unwrap();
                let amount = *attacker.spec.amount(&weapon_id).unwrap();

                for i in 0..amount {
                    let weapon_state = attacker.get_weapon_state(&weapon_id, i);

                    if weapon_state.recharge > 0.0 {
                        weapon_state.recharge -= ctx.delta_time;
                    }

                    let can_fire = weapon_state.recharge <= 0.0;
                    let want_to_fire = targeting.get(&attacker_id);

                    match (can_fire, want_to_fire) {
                        (true, Some(target_id)) => {
                            weapon_state.recharge += weapon.reload;

                            result
                                .entry(*attacker_id)
                                .and_modify(|weapon_fire| weapon_fire.weapons.push(weapon_id))
                                .or_insert(WeaponFire {
                                    weapons: vec![weapon_id],
                                    target_id: *target_id,
                                });
                        }
                        (false, _) => logs.push(CombatLog::Recharging {
                            id: *attacker_id,
                            weapon_id: weapon_id,
                            wait_time: weapon_state.recharge,
                        }),
                        (true, None) => {}
                    }
                }
            }
        }

        result
    }

    fn compute_hits(
        ctx: &CombatContext,
        logs: &mut Vec<CombatLog>,
        fires: HashMap<ShipInstanceId, WeaponFire>,
    ) -> Vec<DamageToApply> {
        let mut damages = vec![];

        for (attacker_id, attacker) in &ctx.ships {
            let fire = match fires.get(attacker_id) {
                Some(fire) => fire,
                None => continue,
            };

            let target_id = fire.target_id;
            let defender = ctx.ships.get(&target_id).unwrap();

            for weapon_id in fire.weapons.iter() {
                let weapon = ctx.components.get(&weapon_id).weapon.as_ref().unwrap();
                let hit_chance = Combat::compute_hit_chance(
                    weapon,
                    attacker.current_stats.speed,
                    defender.current_stats.total_width,
                    defender.current_stats.speed,
                );

                for _ in 0..weapon.rounds {
                    if Combat::roll(hit_chance) {
                        damages.push(DamageToApply {
                            attacker_id: *attacker_id,
                            target_id,
                            amount: weapon.damage,
                            weapon_id: *weapon_id,
                            damage_type: weapon.damage_type.clone(),
                        });
                    } else {
                        logs.push(CombatLog::Miss {
                            id: *attacker_id,
                            target_id,
                            weapon_id: *weapon_id,
                        });
                    }
                }
            }
        }

        damages
    }

    // TODO: the ration should not be speed speed, but tracking speed vs difference in speed
    /// =POW(0.5, B12/A12)+POW(0.1, 100 /C12)
    fn compute_hit_chance(
        _weapon: &Weapon,
        attack_speed: Speed,
        target_width: u32,
        target_speed: Speed,
    ) -> f32 {
        let speed_ration: f32 = 0.5_f32.powf(target_speed.0 / attack_speed.0);
        let size_bonus: f32 = 0.1_f32.powf(100.0 / target_width as f32);
        let value = speed_ration + size_bonus;
        if value < 0.01 || value > 0.99 {
            log::warn!("combat - hit chance {:?}, target {:?}, width {:?}. speed_ration {:?}, size_bonus {:?}, value {:?}", attack_speed, target_speed, target_width, speed_ration, size_bonus, value);
        } else {
            log::debug!("combat - hit chance {:?}, target {:?}, width {:?}. speed_ration {:?}, size_bonus {:?}, value {:?}", attack_speed, target_speed, target_width, speed_ration, size_bonus, value);
        }
        value
    }

    fn roll(chance: f32) -> bool {
        let mut rng = rand::thread_rng();
        let value: f32 = rng.gen();
        value <= chance
    }

    fn search_best_target(
        ctx: &CombatContext,
        attacker_id: ShipInstanceId,
    ) -> Option<ShipInstanceId> {
        let team_id = ctx.ships.get(&attacker_id).unwrap().team_id;

        let mut candidates = ctx
            .ships
            .iter()
            .filter(|(_id, other)| other.team_id != team_id)
            .map(|(id, _other)| (*id, ctx.distances.get(&(attacker_id, *id)).unwrap()))
            .collect::<Vec<_>>();

        candidates.sort_unstable_by(|a, b| a.1.partial_cmp(b.1).unwrap());

        candidates.into_iter().map(|(id, _)| id).next()
    }

    // fn roll_order(ships: &HashMap<ShipInstanceId, &mut ShipInstance>) -> Vec<ShipInstanceId> {
    //     ships.keys().map(|i| *i).collect()
    // }
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
                damage_type: WeaponDamageType::Explosive,
            };

            let hit_chance = Combat::compute_hit_chance(
                &weapon,
                Speed(attack_speed),
                target_width,
                Speed(target_speed),
            );
            assert_eq!(hit_chance, expected);
        }

        test(0.5, 0.5, 10, 0.5);
        test(1.0, 2.0, 10, 0.25);
        test(1.0, 10.0, 10, 0.0009765626);
        test(1.0, 10.0, 100, 0.100976564);
    }
}
