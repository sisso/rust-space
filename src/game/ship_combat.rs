use crate::game::ship_internals::*;

use std::collections::HashMap;

// TODO: ship instances can not be owner
pub struct ShipCombat {
    ship1: ShipInstance,
    ship2: ShipInstance,
    distances: HashMap<(ShipInstanceId, ShipInstanceId), f32>,
    logs: Vec<CombatLog>,
}

pub trait ShipCombatInstanceProvider<'a> {
    fn get_mut(id: ShipInstanceId) -> &'a mut ShipInstance;
    fn get(id: ShipInstanceId) -> &'a ShipInstance;
}

pub enum CombatLog {

}

impl ShipCombat {
    pub fn new(ship1: ShipInstance, ship2: ShipInstance) -> Self {
        ShipCombat { 
            ship1, 
            ship2,
            distances: HashMap::new(),
            logs: vec![]
        }
    }

    pub fn tick(&mut self, delta_time: f32, total_time: f32) -> Vec<CombatLog> {
        let mut log = vec![];
//        ShipCombat::tick_fire(&mut log, delta_time, total_time, &mut self.ship1, &mut self.ship2);
//        ShipCombat::tick_fire(&mut log, delta_time, total_time, &mut self.ship2, &mut self.ship1);
        self.tick_fire(delta_time, total_time, self.ship1.id, self.ship2.id);
        log
    }

    pub fn set_distance(&mut self, id0: ShipInstanceId, id1: ShipInstanceId, distance: f32) {
        self.distances.insert((id0, id1), distance);
        self.distances.insert((id1, id0), distance);
    }

    fn tick_fire(&mut self, delta_time: f32, total_time: f32, attacker_id: ShipInstanceId, target_id: ShipInstanceId) {

    }

//    fn tick_fire(log: &mut Vec<CombatLog>, delta_time: f32, total_time: f32, ship1: &mut ShipInstance, ship2: &mut ShipInstance) {
//
//    }
}
