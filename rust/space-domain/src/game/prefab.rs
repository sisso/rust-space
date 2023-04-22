use crate::game::new_obj::NewObj;
use specs::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Prefab {
    pub obj: NewObj,
}

pub fn find_prefab_by_code<'a>(world: &'a World, code: &str) -> Option<&'a Prefab> {
    let e = super::code::get_entity_by_code(world, code)?;
    let prefab = world.read_storage::<Prefab>().get(e)?;
    Some(prefab)
}
