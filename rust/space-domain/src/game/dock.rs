use crate::game::save::LoadingMapEntity;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Other objects can dock in this object
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct HasDocking {
    pub docked: Vec<Entity>,
}

impl LoadingMapEntity for HasDocking {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.docked
            .iter_mut()
            .for_each(|i| i.map_entity(entity_map));
    }
}
