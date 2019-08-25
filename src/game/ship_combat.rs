use crate::game::ship_internals::*;

use std::collections::{HashMap, HashSet};
use rand::{Rng, RngCore};
use std::borrow::BorrowMut;
use crate::utils::Log;

#[derive(Clone,Debug)]
pub enum CombatLog {
    NoTarget { id: ShipInstanceId },
    Recharging { id: ShipInstanceId, weapon_id: ComponentId, wait_time: f32 },
    Miss { id: ShipInstanceId, target_id: ShipInstanceId, weapon_id: ComponentId},
    Hit { id: ShipInstanceId, target_id: ShipInstanceId, damage: Damage, weapon_id: ComponentId, armor_index: ArmorIndex, hull_damage: bool },
    ComponentDestroy { id: ShipInstanceId, component_id: ComponentId },
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

#[derive(Clone,Debug)]
struct DamageToApply {
    attacker_id: ShipInstanceId,
    target_id: ShipInstanceId,
    amount: Damage,
    weapon_id: ComponentId,
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

        let ships_with_hull_damage: HashSet<ShipInstanceId> =
            logs.iter().flat_map(|i| {
                match i {
                    CombatLog::ComponentDestroy { id, .. } => Some(*id),
                    _ => None,
                }
            }).collect();

        for id in ships_with_hull_damage {
            let ship = ctx.ships.get_mut(&id).unwrap();
            println!("------------------------- UPODATE STAST");
            ship.update_stats(ctx.components);
        }

        logs
    }

    fn apply_damage(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, damage: DamageToApply) {
        let mut ship = ctx.ships.get_mut(&damage.target_id).unwrap();
        let mut rng = rand::thread_rng();
        let armor_width = ship.spec.armor.width;
        let index = rng.next_u32() % armor_width;
        let mut hull_damages = vec![];
        let damage_indexes = Combat::generate_damage_indexes(damage.damage_type, damage.amount, ArmorIndex(index), armor_width);
//        Log::info("combat", &format!("{:?} check damage at {:?}", damage.target_id, damage));
        for damage_index in damage_indexes {
            let hull_damage = Combat::ship_apply_damage(logs, ship, damage_index);
            if hull_damage {
                hull_damages.push(damage_index);
            }

            logs.push(CombatLog::Hit {
                id: damage.attacker_id,
                target_id: damage.target_id,
                damage: damage.amount,
                weapon_id: damage.weapon_id,
                armor_index: damage_index,
                hull_damage: hull_damage
            });
        }

        for hull_index in hull_damages {
            Combat::ship_apply_hulldamage(logs, ctx.components, ship, hull_index);
        }
    }

    fn generate_explosive_damage_indexes(amount: Damage, index: ArmorIndex, armor_width: u32) -> Vec<ArmorIndex> {
        let armor_width = armor_width as i32;
        let mut result: Vec<ArmorIndex> = vec![];

        let mut width = 0;
        let mut left: bool = true;
        let mut max_width = 0;
        for i in 0..amount.0 {
            let mut relative;

            if width == 0 {
                left = true;
                max_width += 1;
                width = max_width;
                relative = 0;
            } else if left {
                left = !left;
                relative = -width;
            } else {
                left = !left;
                relative = width;
                width -= 1;
            }

            let next_index= Combat::normalize_width(index.0 as i32 + relative, armor_width);
            if let Some(next_index) = next_index {
                result.push(ArmorIndex(next_index));
            }
        }

        result
    }

    fn generate_damage_indexes(damage_type: WeaponDamageType, amount: Damage, index: ArmorIndex, armor_width: u32) -> Vec<ArmorIndex> {
        match damage_type {
            WeaponDamageType::Penetration => Combat::generate_penetration_damage_indexes(amount, index,armor_width),
            WeaponDamageType::Explosive => Combat::generate_explosive_damage_indexes(amount, index,armor_width),
        }
    }

    fn generate_penetration_damage_indexes(amount: Damage, index: ArmorIndex, armor_width: u32) -> Vec<ArmorIndex> {
        let index = index.0 as i32;
        let amount = amount.0 as i32;
        let armor_width = armor_width as i32;

        let mut result: Vec<ArmorIndex> = vec![];

        for i in 0..amount {
            let mut j =
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

            let next_index = Combat::normalize_width(j, armor_width);
            if let Some(next_index) = next_index {
                result.push(ArmorIndex(next_index));
            }
        }
        result
    }

    fn normalize_width(value: i32, max: i32) -> Option<u32> {
        if value < 0 || value >= max {
            None
        } else {
            Some(value as u32)
        }
    }

    /// return true if was not absorb by armor
    fn ship_apply_damage(logs: &mut Vec<CombatLog>, ship: &mut ShipInstance, damage_index: ArmorIndex) -> bool {
        let mut i = damage_index;
        for layer in 0..ship.spec.armor.height {
            if !ship.armor_damage.contains(&i) {
//                Log::info("combat", &format!("{:?} check damage at {:?}/{:?} hit armor", ship.id, damage_index, layer));
                ship.armor_damage.insert(i);
                return false;
            }

            i = ArmorIndex(i.0 + ship.spec.armor.width);
        }

//        Log::info("combat", &format!("{:?} check damage at {:?}:{:?} hit hull", ship.id, damage_index, ship.spec.armor.height));
        true
    }

    fn ship_apply_hulldamage(logs: &mut Vec<CombatLog>, components: &Components, ship: &mut ShipInstance, hull_index: ArmorIndex) {
        let mut rng = rand::thread_rng();
        let mut hit = rng.gen_range(0, ship.spec.component_table.total as i32);

//        Log::debug("combat", &format!("{:?} component table, total {:?}, hit {:?}: {:?}", ship.id, ship.spec.component_table.total, hit, ship.spec.component_table));

        for (component_id, width) in ship.spec.component_table.sequence.iter() {
            hit -= *width as i32;

            if hit <= 0 {
//                Log::debug("combat", &format!("{:?} hull hit at component {:?}", ship.id, component_id));

                let total_damage = ship.component_damage.entry(*component_id)
                    .and_modify(|i| i.0 += 1)
                    .or_insert(Damage(1));

                let component = components.get(component_id);
                let total_component_width = component.width * *ship.spec.components.get(component_id).unwrap();
                let total_damage_percent = total_damage.0 as f32 / total_component_width as f32;
                let mut rng = rand::thread_rng();
                let chance = rng.gen::<f32>();

                if chance < total_damage_percent {
                    ship.component_destroyed
                        .entry(*component_id)
                        .and_modify(|amount| amount.0 += 1)
                        .or_insert(Amount(1));

                    logs.push(CombatLog::ComponentDestroy {
                        id: ship.id,
                        component_id: *component_id
                    });
                }

                break;
            }
        }
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
            let value = Combat::generate_penetration_damage_indexes(Damage(damage), ArmorIndex(1), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
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

    #[test]
    fn generate_damage_indexes_penetration_overflow_tests() {
        fn test(index: u32, damage: u32, expected: Vec<u32>) {
            let value = Combat::generate_penetration_damage_indexes(Damage(damage), ArmorIndex(index), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(0, 9, vec![0, 0, 0, 1, 0, 1, 0]);
        test(3, 9, vec![3, 3, 3, 2, 3, 2, 3]);
    }

    #[test]
    fn generate_damage_indexes_explosion_tests() {
        fn test(damage: u32, expected: Vec<u32>) {
            let value = Combat::generate_explosive_damage_indexes(Damage(damage), ArmorIndex(5), 10);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(1, vec![5]);
        test(2, vec![5, 4]);
        test(3, vec![5, 4, 6]);
        test(4, vec![5, 4, 6, 5]);
        test(5, vec![5, 4, 6, 5, 3]);
        test(6, vec![5, 4, 6, 5, 3, 7]);
        test(7, vec![5, 4, 6, 5, 3, 7, 4]);
        test(8, vec![5, 4, 6, 5, 3, 7, 4, 6]);
        test(9, vec![5, 4, 6, 5, 3, 7, 4, 6, 5]);
    }

    #[test]
    fn generate_damage_indexes_explosion_tests_underflow() {
        fn test(index: u32, damage: u32, expected: Vec<u32>) {
            let value = Combat::generate_explosive_damage_indexes(Damage(damage), ArmorIndex(index), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(0, 1, vec![0]);
        test(0, 2, vec![0]);
        test(0, 3, vec![0, 1]);
        test(0, 4, vec![0, 1, 0]);
    }

    #[test]
    fn generate_damage_indexes_explosion_tests_overflow() {
        fn test(index: u32, damage: u32, expected: Vec<u32>) {
            let value = Combat::generate_explosive_damage_indexes(Damage(damage), ArmorIndex(index), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(3, 1, vec![3]);
        test(3, 2, vec![3, 2]);
        test(3, 3, vec![3, 2]);
        test(3, 4, vec![3, 2, 3]);
    }
}
