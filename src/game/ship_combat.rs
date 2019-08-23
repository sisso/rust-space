use crate::game::ship_internals::*;

use std::collections::HashMap;
use rand::{Rng, RngCore};

#[derive(Clone,Debug)]
pub enum CombatLog {
    NoTarget { id: ShipInstanceId },
    Recharging { id: ShipInstanceId, weapon_id: ComponentId, wait_time: f32 },
    Miss { id: ShipInstanceId, target_id: ShipInstanceId, weapon_id: ComponentId},
    Hit { id: ShipInstanceId, target_id: ShipInstanceId, damage: Damage, weapon_id: ComponentId },
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

struct DamageToApply {
    target_id: ShipInstanceId,
    amount: Damage,
    damage_type: WeaponDamageType,
}

impl Combat {

    pub fn execute(ctx: &mut CombatContext) -> Vec<CombatLog> {
        let mut logs = vec![];
        let mut damages = vec![];
        let ship_sequence = Combat::roll_order(&ctx.ships);

        for ship_id in ship_sequence {
            Combat::execute_attack(ctx, &mut logs, &mut damages, ship_id);
        }

        for damage in damages {
            Combat::apply_damage(ctx, &mut logs, damage);
        }

        logs
    }

    fn apply_damage(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, damage: DamageToApply) {
        let mut ship = ctx.ships.get(&damage.target_id).unwrap();
        let mut rng = rand::thread_rng();
        let armor_width = ship.spec.armor.width;
        let index = rng.next_u32() % armor_width;
        let mut hull_damages = vec![];
        let damage_indexes = Combat::generate_damage_indexes(&damage.damage_type, &damage.amount, index, armor_width);
        for damage_index in damage_indexes {
            if Combat::ship_apply_damage(logs, ship, damage_index) {
                hull_damages.push(damage_index);
            }
        }

        for hull_index in hull_damages {
            Combat::ship_apply_hulldamage(logs, ship, hull_index);
        }
    }

    // TODO: impl
    fn generate_damage_indexes(damage_type: &WeaponDamageType, amount: &Damage, index: u32, armor_width: u32) -> Vec<u32> {
        let mut result = vec![];
        println!("start {:?}", amount);

        match damage_type {
            WeaponDamageType::Explosive => {
            },
            WeaponDamageType::Penetration => {
                for i in 0..amount.0 {
                    let j =
                        if i >= 3 {
                            if i % 3 == 0 {
                                index - 1
                            } else if (i - 1) % 3 == 0 {
                                index + 1
                            } else {
                                index
                            }
                        } else {
                            index
                        };

                    result.push(j);
                }
            },
        }
        result
    }

    /// return true if was not absorb by armor
    // TODO: impl
    fn ship_apply_damage(logs: &mut Vec<CombatLog>, ship: &&mut ShipInstance, damage_index: u32) -> bool {
        false
    }

    // TODO: remove double reference
    fn ship_apply_hulldamage(logs: &mut Vec<CombatLog>, ship: &&mut ShipInstance, hull_index: u32) {
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
        let mut attacker = ctx.ships.get_mut(&attacker_id).unwrap();
        let weapons = attacker.spec.find_weapons(ctx.components);

        for weapon_id in weapons {
            let amount = *attacker.spec.amount(&weapon_id).unwrap();

            for i in 0..amount {
                let weapon_state = attacker.get_weapon_state(&weapon_id, i);

                if weapon_state.recharge > 0.0 {
                    weapon_state.recharge -= ctx.delta_time;
                }

                let can_fire = weapon_state.recharge <= 0.0;
                if can_fire {
                    let weapon = ctx.components.get(&weapon_id).weapon.as_ref().unwrap();
                    weapon_state.recharge += weapon.reload;

                    for _ in 0..weapon.rounds {
                        let hit_chance = Combat::compute_hit_chance(attacker_id, target_id);

                        if Combat::roll(hit_chance) {
                            logs.push(CombatLog::Hit {
                                id: attacker_id,
                                target_id: target_id,
                                damage: weapon.damage,
                                weapon_id: weapon_id
                            });

                            damages.push(DamageToApply {
                                target_id,
                                amount: weapon.damage,
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

    fn compute_hit_chance(attacker_id: ShipInstanceId, target_id: ShipInstanceId) -> f32 {
        0.5
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
    fn generate_damage_indexes_penetration_tests() {
        fn test(damage: u32, expected: Vec<u32>) {
            let value = Combat::generate_damage_indexes(&WeaponDamageType::Penetration, &Damage(damage), 1, 4);
            assert_eq!(value, expected);
        }

        test(1, vec![1]);
        test(2, vec![1, 1]);
        test(3, vec![1, 1, 1]);
        test(4, vec![1, 1, 1, 0]);
        test(5, vec![1, 1, 1, 0, 2]);
        test(6, vec![1, 1, 1, 0, 2, 1]);
        test(7, vec![1, 1, 1, 0, 2, 1, 0]);
        test(8, vec![1, 1, 1, 0, 2, 1, 0, 2]);
        test(9, vec![1, 1, 1, 0, 2, 1, 0, 2, 1]);
    }

//    #[test]
//    fn generate_damage_indexes_penetration_overflow_tests() {
//        fn test(index: u32, damage: u32, expected: Vec<u32>) {
//            let value = Combat::generate_damage_indexes(&WeaponDamageType::Penetration, &Damage(damage), index, 4);
//            assert_eq!(value, expected);
//        }
//
//        test(0, 9, vec![0, 0, 0, 3, 1, 0, 3, 1, 0]);
//        test(3, 9, vec![3, 3, 3, 2, 0, 3, 2, 0, 3]);
//    }
//
//    #[test]
//    fn generate_damage_indexes_explosion_tests() {
//        fn test(index: u32, damage: u32, expected: Vec<u32>) {
//            let value = Combat::generate_damage_indexes(&WeaponDamageType::Penetration, &Damage(1), index, 4);
//            assert_eq!(value, expected);
//        }
//
//        test(0, 1, vec![0]);
//        test(0, 2, vec![0, 3]);
//        test(0, 3, vec![0, 3, 1]);
//        test(0, 4, vec![0, 3, 1, 0]);
//        test(0, 5, vec![0, 3, 1, 0, 3]);
//        test(0, 6, vec![0, 3, 1, 0, 3, 1]);
//    }
}
