use crate::game::code;
use crate::game::code::HasCode;
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use specs::prelude::*;

pub type PrefabId = ObjId;

/// Define a NewObj that can easily be builder by the engine when a new object would need to be
/// created
#[derive(Debug, Clone, Component)]
pub struct Prefab {
    pub obj: NewObj,
}

pub fn find_prefab_by_code(world: &World, code: &str) -> Option<Prefab> {
    let e = code::get_entity_by_code(world, code)?;
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

pub fn get_by_id(prefabs: &ReadStorage<'_, Prefab>, prefab_id: PrefabId) -> Option<NewObj> {
    prefabs.get(prefab_id).map(|prefab| prefab.obj.clone())
}
