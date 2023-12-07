use crate::game::code;
use crate::game::code::HasCode;
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use bevy_ecs::bundle::DynamicBundle;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

pub type PrefabId = ObjId;

/// Define a NewObj that can easily be builder by the engine when a new object would need to be
/// created
#[derive(Debug, Clone, Component)]
pub struct Prefab {
    pub obj: NewObj,
    pub shipyard: bool,
    pub build_site: bool,
}

pub fn find_prefab_by_code(
    input: In<(String)>,
    query_codes: Query<(Entity, &HasCode)>,
    query_prefabs: Query<&Prefab>,
) -> Option<Prefab> {
    let prefab_id = code::find_entity_by_code(input, query_codes)?;
    query_prefabs.get(prefab_id).ok().cloned()
}
