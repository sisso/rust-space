use crate::game::ship_internals::*;

pub struct ShipCombat {
    pub ship1: ShipInstance,
    pub ship2: ShipInstance,
}

pub enum CombatLog {

}

impl ShipCombat {
    pub fn new(ship1: ShipInstance, ship2: ShipInstance) -> Self {
        ShipCombat { ship1, ship2 }
    }

    pub fn tick(&mut self, delta_time: f32, total_time: f32) -> Vec<CombatLog> {
        vec![]
    }
}
