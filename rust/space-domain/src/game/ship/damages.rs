use std::collections::{HashMap, HashSet};

use rand::{Rng, RngCore};

use crate::game::ship::ship_combat::CombatLog;

use super::ship_internals::*;

#[derive(Clone, Debug)]
pub struct DamageToApply {
    pub attacker_id: ShipInstanceId,
    pub target_id: ShipInstanceId,
    pub amount: Damage,
    pub weapon_id: ComponentId,
    pub damage_type: WeaponDamageType,
}

pub fn apply_damages(
    components: &Components,
    logs: &mut Vec<CombatLog>,
    ships: &mut HashMap<ShipInstanceId, &mut ShipInstance>,
    damages: Vec<DamageToApply>,
) {
    for damage in damages {
        let ship = ships.get_mut(&damage.target_id).unwrap();
        apply_damage(components, logs, ship, damage);
    }

    let ships_with_hull_damage: HashSet<ShipInstanceId> = logs
        .iter()
        .flat_map(|i| match i {
            CombatLog::ComponentDestroy { id, .. } => Some(*id),
            _ => None,
        })
        .collect();

    for id in ships_with_hull_damage {
        // compute ship destroy
        let ship = ships.get_mut(&id).unwrap();

        let total_hull = ship.spec.get_hull_hp(components);
        let total_hull_damage = ship.get_total_hull_damage();

        if wreck_check(total_hull, total_hull_damage) {
            logs.push(CombatLog::ShipDestroyed { id: ship.id });
            ship.wreck = true;
        }

        // update stats
        ship.update_stats(components);
    }
}

fn apply_damage(
    components: &Components,
    logs: &mut Vec<CombatLog>,
    ship: &mut ShipInstance,
    damage: DamageToApply,
) {
    let mut rng = rand::thread_rng();
    let armor_width = ship.spec.armor.width;
    let index = rng.next_u32() % armor_width;
    let mut hull_damages = vec![];
    let damage_indexes = generate_damage_indexes(
        damage.damage_type,
        damage.amount,
        ArmorIndex(index),
        armor_width,
    );
    //        info!("combat", &format!("{:?} check damage at {:?}", damage.target_id, damage));
    for damage_index in damage_indexes {
        let hull_damage = ship_apply_damage(logs, ship, damage_index);
        if hull_damage {
            hull_damages.push(damage_index);
        }

        logs.push(CombatLog::Hit {
            id: damage.attacker_id,
            target_id: damage.target_id,
            damage: damage.amount,
            weapon_id: damage.weapon_id,
            armor_index: damage_index,
            hull_damage: hull_damage,
        });
    }

    for hull_index in hull_damages {
        ship_apply_hulldamage(logs, components, ship, hull_index);
    }
}

fn wreck_check(total_hull: Hp, total_damage: Damage) -> bool {
    if total_hull.0 / 2 > total_damage.0 {
        debug!(target: "combat", "wreck check not require, hp: {:?} / 2 > damage: {:?}", total_hull, total_damage);
        false
    } else {
        let ration = total_damage.0 as f32 / total_hull.0 as f32;
        let chance = ration.powi(2);
        let mut rng = rand::thread_rng();
        let dice: f32 = rng.gen();
        if chance >= dice {
            debug!(target: "combat", "wreck check success, ship destroy chance: {:?} >= dice: {:?}, ", chance, dice);
            true
        } else {
            debug!(target: "combat", "wreck check fail, ship still functional chance: {:?} >= dice: {:?}, ", chance, dice);
            false
        }
    }
}

fn generate_explosive_damage_indexes(
    amount: Damage,
    index: ArmorIndex,
    armor_width: u32,
) -> Vec<ArmorIndex> {
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

        let next_index = normalize_width(index.0 as i32 + relative, armor_width);

        if let Some(next_index) = next_index {
            result.push(ArmorIndex(next_index));
        }
    }

    result
}

fn generate_damage_indexes(
    damage_type: WeaponDamageType,
    amount: Damage,
    index: ArmorIndex,
    armor_width: u32,
) -> Vec<ArmorIndex> {
    match damage_type {
        WeaponDamageType::Penetration => {
            generate_penetration_damage_indexes(amount, index, armor_width)
        }
        WeaponDamageType::Explosive => {
            generate_explosive_damage_indexes(amount, index, armor_width)
        }
    }
}

fn generate_penetration_damage_indexes(
    amount: Damage,
    index: ArmorIndex,
    armor_width: u32,
) -> Vec<ArmorIndex> {
    let index = index.0 as i32;
    let amount = amount.0 as i32;
    let armor_width = armor_width as i32;

    let mut result: Vec<ArmorIndex> = vec![];

    for i in 0..amount {
        let j = if i >= 3 {
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

        let next_index = normalize_width(j, armor_width);
        if let Some(next_index) = next_index {
            result.push(ArmorIndex(next_index));
        }
    }
    result
}

/// return true if was not absorb by armor
fn ship_apply_damage(
    logs: &mut Vec<CombatLog>,
    ship: &mut ShipInstance,
    damage_index: ArmorIndex,
) -> bool {
    let mut i = damage_index;
    for layer in 0..ship.spec.armor.height {
        if !ship.armor_damage.contains(&i) {
            //                info!("combat", &format!("{:?} check damage at {:?}/{:?} hit armor", ship.id, damage_index, layer));
            ship.armor_damage.insert(i);
            return false;
        }

        i = ArmorIndex(i.0 + ship.spec.armor.width);
    }

    //        info!("combat", &format!("{:?} check damage at {:?}:{:?} hit hull", ship.id, damage_index, ship.spec.armor.height));
    true
}

fn ship_apply_hulldamage(
    logs: &mut Vec<CombatLog>,
    components: &Components,
    ship: &mut ShipInstance,
    hull_index: ArmorIndex,
) {
    let mut rng = rand::thread_rng();
    let mut hit = rng.gen_range(0, ship.spec.component_table.total as i32);

    //        Log::debug("combat", &format!("{:?} component table, total {:?}, hit {:?}: {:?}", ship.id, ship.spec.component_table.total, hit, ship.spec.component_table));

    for (component_id, width) in ship.spec.component_table.sequence.iter() {
        hit -= *width as i32;

        if hit <= 0 {
            //                Log::debug("combat", &format!("{:?} hull hit at component {:?}", ship.id, component_id));

            let total_damage = ship
                .component_damage
                .entry(*component_id)
                .and_modify(|i| i.0 += 1)
                .or_insert(Damage(1));

            let component = components.get(component_id);
            let total_component_width =
                component.width * *ship.spec.components.get(component_id).unwrap();
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
                    component_id: *component_id,
                });
            }

            break;
        }
    }
}

fn normalize_width(value: i32, max: i32) -> Option<u32> {
    if value < 0 || value >= max {
        None
    } else {
        Some(value as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::Speed;

    use super::*;

    #[test]
    fn generate_damage_indexes_penetration_tests() {
        fn test(damage: u32, expected: Vec<u32>) {
            let value = generate_penetration_damage_indexes(Damage(damage), ArmorIndex(1), 4);
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
            let value = generate_penetration_damage_indexes(Damage(damage), ArmorIndex(index), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(0, 9, vec![0, 0, 0, 1, 0, 1, 0]);
        test(3, 9, vec![3, 3, 3, 2, 3, 2, 3]);
    }

    #[test]
    fn generate_damage_indexes_explosion_tests() {
        fn test(damage: u32, expected: Vec<u32>) {
            let value = generate_explosive_damage_indexes(Damage(damage), ArmorIndex(5), 10);
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
            let value = generate_explosive_damage_indexes(Damage(damage), ArmorIndex(index), 4);
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
            let value = generate_explosive_damage_indexes(Damage(damage), ArmorIndex(index), 4);
            let value: Vec<u32> = value.into_iter().map(|i| i.0).collect();
            assert_eq!(value, expected);
        }

        test(3, 1, vec![3]);
        test(3, 2, vec![3, 2]);
        test(3, 3, vec![3, 2]);
        test(3, 4, vec![3, 2, 3]);
    }
}
