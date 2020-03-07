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
use crate::game::dock::HasDock;
use crate::game::order::{Order, Orders};

pub struct CommandMineSystem;

#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    locations: ReadStorage<'a, Location>,
    commands: WriteStorage<'a, Command>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    action_extract: ReadStorage<'a, ActionExtract>,
    action_request: WriteStorage<'a, ActionRequest>,
    extractable: ReadStorage<'a, Extractable>,
    docks: ReadStorage<'a, HasDock>,
    orders: ReadStorage<'a, Orders>,
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
            &mut data.commands,
            &data.cargos,
            !&data.navigation,
            !&data.action_extract,
            &data.locations,
        )
            .join()
        {
            // re-assign to stop Intelij to complain
            let command: &mut Command = command;

            let command = match command {
                Command::Mine(mine) => mine,
                _ => continue,
            };

            if cargo.is_full() {
                // deliver cargo

                // get or search for a target
                let target_id = match command.deliver_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        let wares_to_deliver: Vec<&WareId> = cargo.get_wares().collect();

                        match search_deliver_target(sectors_index, entity, sector_id, &data.orders, &wares_to_deliver) {
                            Some(target_id) => {
                                command.deliver_target_id = Some(target_id);
                                target_id
                            },
                            None => {
                                warn!("{:?} can not find a cargo for deliver", entity);
                                continue;
                            }
                        }
                    }
                };

                if location.as_docked() == Some(target_id) {
                    debug!("{:?} cargo full, transferring cargo to station {:?}", entity, target_id);
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
    orders: &ReadStorage<Orders>,
    wares_to_deliver: &Vec<&WareId>,
) -> Option<ObjId> {
    // find nearest deliver
    let candidates = sectors_index.search_nearest_stations(sector_id);
    candidates.iter()
        .flat_map(|(sector_id, candidate_id)| {
            let has_request =
                orders.get(*candidate_id)
                    .map(|orders| {
                        orders.ware_requests().iter().any(|ware_id| {
                            wares_to_deliver.contains(&ware_id)
                        })
                    })
                    .unwrap_or(false);

            if has_request {
                Some(*candidate_id)
            } else {
                None
            }
        })
        .next()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery;
    use crate::game::wares::WareId;
    use crate::test::test_system;
    use specs::DispatcherBuilder;
    use crate::game::order::Order;

    struct SceneryRequest {}

    #[derive(Debug)]
    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
        station: ObjId,
        ware_id: WareId,
    }

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_scenery = crate::game::sectors::test_scenery::setup_sector_scenery(world);

        let ware_id = world.create_entity().build();

        let extractable = Extractable { ware_id };

        let asteroid = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            })
            .with(extractable)
            .build();

        let station = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            })
            .with(HasDock)
            .with(Cargo::new(100.0))
            .with(Orders::new(Order::WareRequest { wares_id: vec![ware_id] }))
            .build();

        let miner = world
            .create_entity()
            .with(Location::Dock {
                docked_id: station,
            })
            .with(Command::mine())
            .with(Cargo::new(10.0))
            .build();

        // inject objects into the location index
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_scenery.sector_0, station);
        entities_per_sector.add_extractable(sector_scenery.sector_0, asteroid);
        world.insert(entities_per_sector);

        let scenery = SceneryResult {
            miner,
            asteroid,
            station,
            ware_id
        };

        debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn remove_station_ware_order(world: &mut World, scenery: &SceneryResult) {
        world.write_storage::<Orders>().borrow_mut().remove(scenery.station).unwrap();
    }

    fn move_miner_to_asteroid(world: &mut World, scenery: &SceneryResult) {
        // move ship to asteroid, should start to mine
        let location_storage = &mut world.write_storage::<Location>();
        let asteroid_location = location_storage.get(scenery.asteroid);
        location_storage.insert(scenery.miner, asteroid_location.unwrap().clone()).unwrap();
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(CommandMineSystem, |world| setup_scenery(world));

        let command_storage = world.read_component::<Command>();
        let command = command_storage.get(scenery.miner).and_then(|i| i.as_mine());
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
            move_miner_to_asteroid(world, &scenery);
            scenery
        });
        let action = world.read_storage::<ActionRequest>().get(scenery.miner).cloned();
        assert!(action.is_some());
        assert_eq!(action.unwrap().0, Action::Extract { target_id: scenery.asteroid, ware_id: scenery.ware_id });
    }

    #[test]
    fn test_command_mine_should_be_wait_if_cargo_is_full_and_has_no_target_station() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            remove_station_ware_order(world, &scenery);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(scenery.ware_id, 10.0);

            scenery
        });

        let nav_request_storage = world.read_storage::<NavRequest>();

        match nav_request_storage.get(scenery.miner) {
            None => {
                // nothing, waiting until something happens
            },
            other => panic!("unexpected nav request {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_navigate_to_station_when_cargo_is_full() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            move_miner_to_asteroid(world, &scenery);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(scenery.ware_id, 10.0);

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
    fn test_command_mine_should_not_deliver_cargo_to_station_if_not_require() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            remove_station_ware_order(world, &scenery);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(scenery.ware_id, 10.0);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();
        let miner_cargo = cargo_storage.get(scenery.miner).unwrap();
        assert_eq!(10.0, miner_cargo.get_current());
    }

    #[test]
    fn test_command_mine_should_deliver_cargo_to_station_when_docked() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(scenery.ware_id, 10.0);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();

        let miner_cargo = cargo_storage.get(scenery.miner).unwrap();
        assert_eq!(0.0, miner_cargo.get_current());

        let station_cargo = cargo_storage.get(scenery.station).unwrap();
        assert_eq!(10.0, station_cargo.get_current());
    }
}
