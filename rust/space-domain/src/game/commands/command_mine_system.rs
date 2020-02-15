use shred::{Read, ResourceId, SystemData, World, Write};
///
/// System plans:
///
/// - for miner command without target, find a target
/// - for miner command with target without nav, if near target, mine
/// - for miner command with target without nave, if far away, create navigation to target
/// - for miner command with target without nav and mine action, if full, move back
///
/// - search for target for non assigned miners
/// - create navigation plans for new miners
/// - start mine for miners that arrival at target
/// - trace back plan for miners that have full cargo
/// - deliver cargo
///
///
use specs::prelude::*;
use specs_derive::*;

use super::*;
use crate::game::extractables::Extractable;
use crate::game::locations::{EntityPerSectorIndex, Location};
use crate::game::navigations::{NavRequest, Navigation, NavigationMoveTo, Navigations};
use crate::game::wares::Cargo;
use std::borrow::{Borrow, BorrowMut};

pub struct CommandMineSystem;

#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    locations: ReadStorage<'a, Location>,
    commands_mine: WriteStorage<'a, CommandMine>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    action_extract: ReadStorage<'a, ActionExtract>,
    action_request: WriteStorage<'a, ActionRequest>,
}

impl<'a> System<'a> for CommandMineSystem {
    type SystemData = CommandMineData<'a>;

    fn run(&mut self, mut data: CommandMineData) {
        trace!("running");

        let sectors_index = data.sector_index.borrow();
        let locations = data.locations.borrow();
        let mut cargo_transfers = vec![];

        // find mine commands without action or navigation
        for (entity, command, cargo, _, _, location) in (
            &*data.entities,
            &mut data.commands_mine,
            &data.cargos,
            !&data.navigation,
            !&data.action_extract,
            &data.locations,
        )
            .join()
        {
            if cargo.is_full() {
                let target_id = match command.deliver_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        let target_id = search_deliver_target(sectors_index, entity, sector_id);
                        command.deliver_target_id = Some(target_id);
                        target_id
                    }
                };

                if location.as_docked() == Some(target_id) {
                    cargo_transfers.push((entity, target_id));
                } else {
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveAndDockAt { target_id });
                }
            } else {
                // mine for non full cargo
                let target_id = match command.mine_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        let target_id = search_mine_target(sectors_index, entity, sector_id);
                        command.mine_target_id = Some(target_id);
                        target_id
                    }
                };

                // navigate to mine
                let target_location = locations.get(target_id).unwrap();
                if Locations::is_near(location, &target_location) {
                    data.action_request
                        .borrow_mut()
                        .insert(entity, ActionRequest(Action::Extract { target_id }));
                } else {
                    // move to target
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveToTarget { target_id });
                }
            }
        }

        // transfer all cargos
        let cargos = data.cargos.borrow_mut();
        for (from_id, to_id) in cargo_transfers {
            Cargos::move_all(cargos, from_id, to_id);
        }
    }
}

fn search_mine_target(
    sectors_index: &EntityPerSectorIndex,
    entity: Entity,
    sector_id: SectorId,
) -> ObjId {
    // find nearest extractable
    let candidates = sectors_index.search_nearest_extractable(sector_id);
    let target_id = candidates.iter().next().unwrap();
    target_id.1
}

fn search_deliver_target(
    sectors_index: &EntityPerSectorIndex,
    entity: Entity,
    sector_id: SectorId,
) -> ObjId {
    // find nearest deliver
    let candidates = sectors_index.list_stations();
    let target_id = candidates.iter().next().unwrap();
    target_id.1
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery;
    use crate::game::sectors::test_scenery::{SECTOR_0, SECTOR_1};
    use crate::game::wares::WareId;
    use crate::test::test_system;
    use specs::DispatcherBuilder;

    struct SceneryRequest {}

    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
    }

    const WARE_0: WareId = WareId(0);
    const EXTRACTABLE: Extractable = Extractable {
        ware_id: WARE_0,
        time: DeltaTime(1.0),
    };

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let asteroid = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_1,
            })
            // TODO
//            .with(EXTRACTABLE)
            .build();

        let miner = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_0,
            })
            .with(CommandMine::new())
            .with(Cargo::new(10.0))
            .build();

        // TODO: use index
        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        world.insert(entitys_per_sector);

        SceneryResult { miner, asteroid }
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(CommandMineSystem, |world| setup_scenery(world));

        let command_storage = world.read_component::<CommandMine>();
        let command = command_storage.get(scenery.miner);
        match command {
            Some(command) => {
                assert_eq!(command.mine_target_id, Some(scenery.asteroid));
            }
            None => {
                panic!("miner has no commandmine");
            }
        }

        let request_storage = world.read_storage::<NavRequest>();
        match request_storage.get(scenery.miner) {
            Some(NavRequest::MoveToTarget { target_id: target }) => {
                assert_eq!(target.clone(), scenery.asteroid);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_command_mine_should_mine() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let asteroid = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(0.0, 0.0),
                    sector_id: SECTOR_0,
                })
//                .with(EXTRACTABLE)
                .build();

            let miner = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(0.01, 0.0),
                    sector_id: SECTOR_0,
                })
                .with(CommandMine::new())
                .build();

            SceneryResult { miner, asteroid }
        });

        //        world.read_component::<CommandMine>
    }
}
