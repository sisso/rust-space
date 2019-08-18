use crate::game::ship_internals::ShipInternal;

pub struct ShipCombat {
    pub ship1: ShipInternal,
    pub ship2: ShipInternal,
}

pub enum CombatLog {

}

impl ShipCombat {
    pub fn new(ship1: ShipInternal, ship2: ShipInternal) -> Self {
        ShipCombat { ship1, ship2 }
    }

    pub fn tick(&mut self) -> Vec<CombatLog> {
        vec![]
    }
}
