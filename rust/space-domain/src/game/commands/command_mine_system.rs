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
use crate::game::wares::{Cargo, WareId};
use std::borrow::{Borrow, BorrowMut};
use crate::game::station::Station;

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
    extractable: ReadStorage<'a, Extractable>,
    stations: ReadStorage<'a, Station>,
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
                    debug!("{:?} cargo full, transfering cargo to station {:?}", entity, target_id);
                    cargo_transfers.push((entity, target_id));
                } else {
                    debug!("{:?} cargo full, command to navigate to station {:?}", entity, target_id);
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveAndDockAt { target_id })
                        .unwrap();
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
                    // find extractable ware id
                    let ware_id = data.extractable.get(target_id).unwrap().ware_id;

                    debug!("{:?} arrive at extractable {:?}, start extraction of {:?}", entity, target_id, ware_id);

                    data.action_request
                        .borrow_mut()
                        .insert(entity, ActionRequest(Action::Extract { target_id, ware_id }))
                        .unwrap();
                } else {
                    debug!("{:?} command to move to extractable {:?}", entity, target_id);
                    // move to target
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveToTarget { target_id })
                        .unwrap();
                }
            }
        }

        // transfer all cargos
        let cargos = data.cargos.borrow_mut();
        for (from_id, to_id) in cargo_transfers {
            let transfer = Cargos::move_all(cargos, from_id, to_id);
            info!("{:?} transfer {:?} to {:?}", from_id, transfer, to_id);
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
    let candidates = sectors_index.search_nearest_stations(sector_id);
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
    use crate::game::station::Station;

    struct SceneryRequest {}

    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
        station: ObjId,
    }

    const WARE_0: WareId = WareId(0);
    const EXTRACTABLE: Extractable = Extractable {
        ware_id: WARE_0,
    };

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let asteroid = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_1,
            })
            .with(EXTRACTABLE)
            .build();

        let station = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_0,
            })
            .with(Station {})
            .with(Cargo::new(100.0))
            .build();

        let miner = world
            .create_entity()
            .with(Location::Dock {
                docked_id: station,
            })
            .with(CommandMine::new())
            .with(Cargo::new(10.0))
            .build();

        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_stations(SECTOR_1, station);
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        world.insert(entitys_per_sector);

        SceneryResult { miner, asteroid, station }
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
            let scenery = setup_scenery(world);

            // move ship to asteroid, should start to mine
            let location_storage = &mut world.write_storage::<Location>();
            let asteroid_location = location_storage.get(scenery.asteroid);
            location_storage.insert(scenery.miner, asteroid_location.unwrap().clone()).unwrap();

            scenery
        });
        let action = world.read_storage::<ActionRequest>().get(scenery.miner).cloned();
        assert!(action.is_some());
        assert_eq!(action.unwrap().0, Action::Extract { target_id: scenery.asteroid, ware_id: WARE_0 });
    }

    #[test]
    fn test_command_mine_should_navigate_to_station_when_cargo_is_full() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // move ship to asteroid
            let location_storage = &mut world.write_storage::<Location>();
            let asteroid_location = location_storage.get(scenery.asteroid);
            location_storage.insert(scenery.miner, asteroid_location.unwrap().clone()).unwrap();

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(WARE_0, 10.0);

            scenery
        });

        let nav_request_storage = world.read_storage::<NavRequest>();

        match nav_request_storage.get(scenery.miner) {
            Some(NavRequest::MoveAndDockAt{ target_id: target }) => {
                assert_eq!(target.clone(), scenery.station);
            }
            other => panic!("unexpected nav request {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_deliver_cargo_to_station_when_docked() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(WARE_0, 10.0);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();

        let miner_cargo = cargo_storage.get(scenery.miner).unwrap();
        assert_eq!(0.0, miner_cargo.get_current());

        let station_cargo = cargo_storage.get(scenery.station).unwrap();
        assert_eq!(10.0, station_cargo.get_current());
    }
}
