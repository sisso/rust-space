use crate::game::save::LoadingMapEntity;
use crate::game::wares::WareAmount;
use crate::game::work::WorkUnit;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How much cost to build this unit/prefab
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ProductionCost {
    pub cost: Vec<WareAmount>,
    pub work: WorkUnit,
}

impl LoadingMapEntity for ProductionCost {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.cost.map_entity(entity_map);
    }
}
