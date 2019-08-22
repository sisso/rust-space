use crate::game::ship_internals::*;

use std::collections::HashMap;
use rand::Rng;

#[derive(Clone,Debug)]
pub enum CombatLog {
    NoTarget { id: ShipInstanceId },
    Recharging { id: ShipInstanceId, weapon_id: ComponentId },
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

    pub fn execute(ctx: &mut CombatContext) -> Vec<CombatLog> {
        let mut logs = vec![];
        let ship_sequence = Combat::roll_order(&ctx.ships);

        for ship_id in ship_sequence {
            Combat::execute_attack(ctx, &mut logs, ship_id);
        }

        logs
    }

    fn execute_attack(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, attacker_id: ShipInstanceId) {
        let target_id = match Combat::search_best_target(ctx, attacker_id) {
            Some(target_id) => target_id,
            None => {
                logs.push(CombatLog::NoTarget { id: attacker_id });
                return;
            }
        };

        Combat::execute_fire_at(ctx, logs, attacker_id, target_id);
    }

    fn execute_fire_at(ctx: &mut CombatContext, logs: &mut Vec<CombatLog>, attacker_id: ShipInstanceId, target_id: ShipInstanceId) {
        let mut attacker = ctx.ships.get_mut(&attacker_id).unwrap();
        let weapons = attacker.spec.find_weapons(ctx.components);

        for weapon_id in weapons {
            let amount = *attacker.spec.amount(&weapon_id).unwrap();

            for i in 0..amount {
                let weapon_state = attacker.get_weapon_state(&weapon_id, i);
                weapon_state.recharge -= ctx.delta_time;

                let can_fire = weapon_state.recharge <= 0.0;
                if can_fire {
                    let weapon = ctx.components.get(&weapon_id).weapon.as_ref().unwrap();
                    weapon_state.recharge += weapon.reload;

                    for _ in 0..weapon.rounds {
                        Combat::execute_fire_at_with(logs, attacker_id, weapon_id, target_id, weapon.damage);
                    }
                } else {
                    logs.push(CombatLog::Recharging { id: attacker_id, weapon_id: weapon_id });
                }
            }
        }
    }

    fn execute_fire_at_with(logs: &mut Vec<CombatLog>, attacker_id: ShipInstanceId, weapon_id: ComponentId, target_id: ShipInstanceId, damage: Damage) {
        let hit_chance = Combat::compute_hit_chance(attacker_id, target_id);
        if Combat::roll(hit_chance) {
            logs.push(CombatLog::Hit {
                id: attacker_id,
                target_id: target_id,
                damage: damage,
                weapon_id: weapon_id
            });
        } else {
            logs.push(CombatLog::Miss {
                id: attacker_id,
                target_id: target_id,
                weapon_id: weapon_id
            });
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
