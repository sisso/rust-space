use crate::game::code;
use crate::game::code::HasCode;
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use crate::game::save::MapEntity;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PrefabId = ObjId;

/// Define a NewObj that can easily be builder by the engine when a new object would need to be
/// created
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Prefab {
    pub obj: NewObj,
    pub shipyard: bool,
    pub build_site: bool,
}

pub fn find_prefab_by_code(
    input: In<String>,
    query_codes: Query<(Entity, &HasCode)>,
    query_prefabs: Query<&Prefab>,
) -> Option<Prefab> {
    let prefab_id = code::find_entity_by_code(input, query_codes)?;
    query_prefabs.get(prefab_id).ok().cloned()
}

impl MapEntity for Prefab {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.obj.map_entity(entity_map);
    }
}
