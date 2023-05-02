use crate::game::new_obj::NewObj;
use specs::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Prefab {
    pub obj: NewObj,
}

pub fn find_prefab_by_code(world: &World, code: &str) -> Option<Prefab> {
    let e = super::code::get_entity_by_code(world, code)?;
    let prefabs = world.read_storage::<Prefab>();
    let prefab = prefabs.get(e)?;
    Some(prefab.clone())
}
