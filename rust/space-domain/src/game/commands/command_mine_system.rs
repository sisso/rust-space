///
/// System plans:
///
/// - search for target for non assigned miners
/// - create navigation plans for new miners
/// - start mine for miners that arrival at target
/// - trace back plan for miners that have full cargo
/// - deliver cargo
///
///

use specs::prelude::*;
use shred::{Read, ResourceId, SystemData, World, Write};
use specs_derive::*;

use super::*;
use crate::game::locations::{LocationDock, EntityPerSectorIndex, LocationSector};
use std::borrow::{Borrow, BorrowMut};
use crate::game::extractables::Extractable;
use crate::game::navigations::{Navigation, NavigationMoveTo, Navigations, NavRequest};

/// For miners without target, search nearest one and create a navigation request
pub struct SearchMineTargetsSystem;

#[derive(SystemData)]
pub struct SearchMineTargetsData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    locations_sector_id: ReadStorage<'a, LocationSector>,
    commands_mine: WriteStorage<'a, CommandMine>,
    commands_mine_target: WriteStorage<'a, CommandMineTarget>,
    nav_request: WriteStorage<'a, NavRequest>,
}

impl<'a> System<'a> for SearchMineTargetsSystem {
    type SystemData = SearchMineTargetsData<'a>;

    fn run(&mut self, mut data: SearchMineTargetsData) {
        use specs::Join;

        // search extractable
        let mut extractables = vec![];

        for (entity, _) in (&data.entities, &data.extractables).join() {
            extractables.push(entity);
        }

        let mut selected = vec![];

        for (entity, _, _, location_sector_id) in (&data.entities, &data.commands_mine, !&data.commands_mine_target, &data.locations_sector_id).join() {
            let sector_id = location_sector_id.sector_id;

            // search for nearest?
            let target: &ObjId = extractables.iter().next().unwrap();

            // set mine command
            let command = CommandMineTarget {
                target_obj_id: target.clone()
            };

            let request = NavRequest::MoveToTarget {
                target: target.clone()
            };

            selected.push((entity, command, request));
        }

        for (entity, state, request) in selected {
            let _ = data.commands_mine_target.insert(entity, state).unwrap();
            let _ = data.nav_request.insert(entity, request).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery;
    use crate::game::sectors::test_scenery::{SECTOR_0, SECTOR_1};
    use specs::DispatcherBuilder;
    use crate::game::wares::WareId;
    use crate::game::locations::LocationSector;
    use crate::test::test_system;

    struct SceneryRequest {
    }

    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
    }

    const WARE_0: WareId = WareId(0);
    const EXTRACTABLE: Extractable = Extractable { ware_id: WARE_0, time: DeltaTime(1.0) };

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let asteroid =
            world.create_entity()
                .with(LocationSector { sector_id: SECTOR_1 })
                .with(EXTRACTABLE)
                .build();

        let miner =
            world.create_entity()
                .with(LocationSector { sector_id: SECTOR_0 })
                .with(CommandMine {})
                .build();

        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        world.insert(entitys_per_sector);
        
        SceneryResult {
            miner,
            asteroid,
        }
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(SearchMineTargetsSystem, |world| {
            setup_scenery(world)
        });

        let command_storage = world.read_component::<CommandMineTarget>();
        let command = command_storage.get(scenery.miner);
        match command {
            Some(command) => {
                assert_eq!(command.target_obj_id, scenery.asteroid);
            },
            None => {
                panic!("miner has no commandmine");
            }
        }

        let request_storage = world.read_storage::<NavRequest>();
        match request_storage.get(scenery.miner) {
            Some(NavRequest::MoveToTarget { target }) => {
                assert_eq!(target.clone(), scenery.asteroid);
            },
            _ => panic!(),
        }
    }
}
