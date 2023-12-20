use crate::game::save::MapEntity;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::game::wares::{ResourceAccessibility, WareId};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Extractable {
    pub ware_id: WareId,
    pub accessibility: ResourceAccessibility,
}

impl MapEntity for Extractable {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.ware_id = entity_map[&self.ware_id];
    }
}
