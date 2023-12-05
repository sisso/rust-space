use crate::game::code;
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
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

pub fn find_prefab_by_code<'a>(world: &'a mut World, code: &str) -> Option<&'a Prefab> {
    let prefab_id = world.run_system_once_with(code, code::find_entity_by_code);
    world.get::<Prefab>(prefab_id)
}
