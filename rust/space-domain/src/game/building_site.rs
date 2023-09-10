use crate::game::locations::Location;
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::wares::{Cargo, WareAmount};
use specs::prelude::*;

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

pub struct BuildingSystem;

impl<'a> System<'a> for BuildingSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Location>,
        WriteStorage<'a, BuildingSite>,
        WriteStorage<'a, Cargo>,
        ReadStorage<'a, Prefab>,
        Read<'a, LazyUpdate>,
    );

    /// check if all required wares in building site is in place, if so, create the new prafabe in
    /// same location and destroy teh building site.
    fn run(
        &mut self,
        (entities, locations, buildings, mut cargos, prefabs, lazy): Self::SystemData,
    ) {
        log::trace!("running");

        let complete: Vec<_> = (&entities, &buildings, &mut cargos)
            .join()
            .flat_map(|(e, building, cargo)| {
                if cargo.remove_all(&building.input).is_ok() {
                    Some((e, building.prefab_id))
                } else {
                    None
                }
            })
            .collect();

        for (e, prefab_id) in complete {
            log::debug!(
                "building site {:?} complete, creating prefab_id {:?}",
                e,
                prefab_id
            );
            let mut new_obj = match prefabs.get(prefab_id) {
                None => {
                    log::warn!("fail to find prefab_id {:?}, ignoring", prefab_id);
                    continue;
                }
                Some(prefab) => prefab.obj.clone(),
            };

            new_obj.location = locations.get(e).cloned();
            lazy.create_entity(&entities).with(new_obj).build();
            _ = entities.delete(e);
        }
    }
}
