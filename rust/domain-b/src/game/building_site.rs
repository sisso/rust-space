use crate::game::loader::Loader;
use crate::game::locations::LocationSpace;
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::wares::{Cargo, WareAmount};
use bevy_ecs::prelude::*;

/// place in space where some prefab is building, once all input resources are there, the prefab is
/// created and the building site removed.
///
/// it should create income order
///
/// expected components: Cargo, Location
#[derive(Clone, Debug, Component)]
pub struct BuildingSite {
    pub prefab_id: PrefabId,
    pub input: Vec<WareAmount>,
}

/// check if all required wares in building site is in place, if so, create the new prafabe in
/// same location and destroy teh building site.
fn system_building_site(
    mut commands: Commands,
    mut query: Query<(Entity, &LocationSpace, &BuildingSite, &mut Cargo)>,
    query_prefabs: Query<&Prefab>,
) {
    log::trace!("running");

    for (obj_id, loc, building_site, mut cargo) in &mut query {
        if cargo.remove_all_or_none(&building_site.input).is_err() {
            continue;
        }

        let mut new_obj = match query_prefabs.get(building_site.prefab_id).ok() {
            None => {
                log::warn!(
                    "fail to find prefab_id {:?}, ignoring",
                    building_site.prefab_id
                );
                continue;
            }
            Some(prefab) => prefab.obj.clone(),
        };

        new_obj.location_space = Some(loc.clone());

        let new_obj_id = Loader::add_object(&mut commands, &new_obj);

        log::debug!(
            "building site {:?} complete, creating prefab_id {:?} with obj_id {:?}",
            obj_id,
            building_site.prefab_id,
            new_obj_id
        );

        commands.entity(obj_id).despawn();
    }
}
