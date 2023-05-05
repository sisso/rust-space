use crate::game::code;
use crate::game::code::HasCode;
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

pub fn find_new_obj_by_code(
    entities: &Entities,
    codes: &ReadStorage<'_, HasCode>,
    prefabs: &ReadStorage<'_, Prefab>,
    code: &str,
) -> Option<NewObj> {
    code::find(entities, codes, code)
        .and_then(|entity_id| prefabs.get(entity_id))
        .map(|prefab| prefab.obj.clone())
}
