use crate::game_api::Id;
use crate::utils::encode_entity;
use godot::prelude::*;
use space_domain::game::shipyard::{ProductionOrder, Shipyard};

#[derive(Clone, Debug, GodotClass)]
#[class(no_init)]
pub struct ShipyardInfo {
    pub shipyard: Shipyard,
}

#[godot_api]
impl ShipyardInfo {
    #[func]
    pub fn has_current_order(&self) -> bool {
        self.shipyard.is_producing()
    }

    #[func]
    pub fn get_current_order(&self) -> Id {
        encode_entity(
            self.shipyard
                .get_producing()
                .expect("has_current_order was not checked first"),
        )
    }

    #[func]
    pub fn has_next_order(&self) -> bool {
        match self.shipyard.get_production_order() {
            ProductionOrder::Next(_) => true,
            _ => false,
        }
    }

    #[func]
    pub fn get_next_order(&self) -> Id {
        match self.shipyard.get_production_order() {
            ProductionOrder::Next(id) => encode_entity(id),
            _ => panic!("unexpected request, has_next_order was not called first"),
        }
    }
}
