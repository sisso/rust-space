use shred::{Read, ResourceId, SystemData, World};
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

use super::*;
use crate::game::dock::HasDock;
use crate::game::extractables::Extractable;
use crate::game::locations::{EntityPerSectorIndex, Location};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::order::Orders;
use crate::game::wares::{Cargo, WareId};
use std::borrow::BorrowMut;

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
        log::trace!("running");

        let sectors_index = &data.sector_index;
        let locations = &data.locations;
        let mut cargo_transfers = vec![];

        let mut already_targets: HashMap<ObjId, u32> = HashMap::new();

        // collect all already target
        for command in (&data.commands).join() {
            match command {
                Command::Mine(mine) => {
                    for target_id in &mine.mine_target_id {
                        *already_targets.entry(*target_id).or_insert(0) += 1;
                    }
                }
                _ => {}
            };
        }

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

                // get or search for a target if not yet defined
                let target_id = match command.deliver_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        let wares_to_deliver: Vec<WareId> = cargo.get_wares().collect();

                        // TODO: build list of delivers
                        match search_orders_target(
                            sectors_index,
                            sector_id,
                            &data.orders,
                            Some(&wares_to_deliver),
                            Vec::new(),
                            false,
                        ) {
                            Some((target_id, _wares)) => {
                                command.deliver_target_id = Some(target_id);
                                command.mine_target_id = None;
                                target_id
                            }
                            None => {
                                log::warn!("{:?} can not find a cargo for deliver", entity);
                                continue;
                            }
                        }
                    }
                };

                if location.as_docked() == Some(target_id) {
                    let target_has_space = !data
                        .cargos
                        .get(target_id)
                        .expect("target has no cargo")
                        .is_full();

                    if target_has_space {
                        log::debug!(
                            "{:?} cargo full, transferring cargo to station {:?}",
                            entity,
                            target_id,
                        );
                        cargo_transfers.push((entity, target_id));
                    } else {
                        log::debug!(
                            "{:?} cargo full, stations {:?} is full, waiting",
                            entity,
                            target_id,
                        );
                    }
                } else {
                    log::debug!(
                        "{:?} cargo full, command to navigate to station {:?}",
                        entity,
                        target_id,
                    );
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

                        let target_id =
                            search_mine_target(sectors_index, &already_targets, entity, sector_id);
                        command.mine_target_id = Some(target_id);
                        command.deliver_target_id = None;

                        *already_targets.entry(target_id).or_insert(0) += 1;

                        target_id
                    }
                };

                // navigate to mine
                let target_location = locations.get(target_id).unwrap();
                if Locations::is_near(location, &target_location) {
                    // find extractable ware id
                    let ware_id = data.extractable.get(target_id).unwrap().ware_id;

                    log::debug!(
                        "{:?} arrive at extractable {:?}, start extraction of {:?}",
                        entity,
                        target_id,
                        ware_id,
                    );

                    data.action_request
                        .borrow_mut()
                        .insert(
                            entity,
                            ActionRequest(Action::Extract { target_id, ware_id }),
                        )
                        .unwrap();
                } else {
                    log::debug!(
                        "{:?} command to move to extractable {:?}",
                        entity,
                        target_id,
                    );
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
            log::info!("{:?} transfer {:?} to {:?}", from_id, transfer, to_id);
        }
    }
}

fn search_mine_target(
    sectors_index: &EntityPerSectorIndex,
    already_targets: &HashMap<ObjId, u32>,
    _entity: Entity,
    sector_id: SectorId,
) -> ObjId {
    // find nearest extractable
    let mut candidates = sectors_index
        .search_nearest_extractable(sector_id)
        .map(|(_, distance, obj_id)| {
            let count = already_targets.get(&obj_id).cloned().unwrap_or(0);
            let score = count * 10 + distance * 11;
            (score, obj_id)
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|(score, _id)| *score);

    // search first
    let (_score, target_id) = candidates.iter().next().expect("target mine not found");
    *target_id
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::order::Order;

    use crate::game::sectors::test_scenery::SectorScenery;
    use crate::game::wares::WareId;
    use crate::test::{init_log, test_system};

    struct SceneryRequest {}

    #[derive(Debug)]
    struct SceneryResult {
        sector_scenery: SectorScenery,
        miner: ObjId,
        asteroid: ObjId,
        station: ObjId,
        ware_id: WareId,
    }

    fn create_asteroid(world: &mut World, location: Location, ware_id: WareId) -> ObjId {
        world
            .create_entity()
            .with(location)
            .with(Extractable { ware_id })
            .build()
    }

    fn create_miner(world: &mut World, location: Location) -> ObjId {
        world
            .create_entity()
            .with(location)
            .with(Command::mine())
            .with(Cargo::new(10.0))
            .build()
    }

    /// Setup a asteroid in sector 0, a mine station in sector 1, a miner docked in the station
    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_scenery = crate::game::sectors::test_scenery::setup_sector_scenery(world);

        let ware_id = world.create_entity().build();

        let asteroid = create_asteroid(
            world,
            Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            },
            ware_id,
        );

        let station = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            })
            .with(HasDock)
            .with(Cargo::new(100.0))
            .with(Orders::new(Order::WareRequest {
                wares_id: vec![ware_id],
            }))
            .build();

        let miner = create_miner(world, Location::Dock { docked_id: station });

        // inject objects into the location index
        // TODO: how to test it easy?
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_scenery.sector_0, station);
        entities_per_sector.add_extractable(sector_scenery.sector_0, asteroid);
        world.insert(entities_per_sector);

        let scenery = SceneryResult {
            sector_scenery,
            miner,
            asteroid,
            station,
            ware_id,
        };

        log::debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn remove_station_ware_order(world: &mut World, scenery: &SceneryResult) {
        world
            .write_storage::<Orders>()
            .borrow_mut()
            .remove(scenery.station)
            .unwrap();
    }

    fn move_miner_to_asteroid(world: &mut World, scenery: &SceneryResult) {
        // move ship to asteroid, should start to mine
        let location_storage = &mut world.write_storage::<Location>();
        let asteroid_location = location_storage.get(scenery.asteroid);
        location_storage
            .insert(scenery.miner, asteroid_location.unwrap().clone())
            .unwrap();
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
        let action = world
            .read_storage::<ActionRequest>()
            .get(scenery.miner)
            .cloned();
        assert!(action.is_some());

        match action.unwrap().0 {
            Action::Extract { target_id, ware_id } => {
                assert_eq!(target_id, scenery.asteroid);
                assert_eq!(ware_id, scenery.ware_id);
            }

            other => panic!("unexpected {:?}", other),
        }
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
            }
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
            Some(NavRequest::MoveAndDockAt { target_id: target }) => {
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

    #[test]
    fn test_command_mine_should_wait_if_target_station_is_full() {
        init_log();

        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner).unwrap();
            cargo.add_to_max(scenery.ware_id, 10.0);

            // fill station to max
            let cargo = cargo_storage.get_mut(scenery.station).unwrap();
            cargo.add_to_max(scenery.ware_id, 100.0);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();

        let miner_cargo = cargo_storage.get(scenery.miner).unwrap();
        assert_eq!(10.0, miner_cargo.get_current());

        let station_cargo = cargo_storage.get(scenery.station).unwrap();
        assert_eq!(100.0, station_cargo.get_current());
    }

    #[test]
    fn test_command_mine_should_split_mining_targets() {
        /// setup a scenery with 2 steroid in same sector with miners and another in neighbor sector
        /// execute for X miner and check expectation.
        fn execute(
            miners_count: usize,
            expect_targets_sector_0: u32,
            expect_targets_sector_1: u32,
        ) {
            let (world, (_scenery, asteroids, miners)) = test_system(CommandMineSystem, |world| {
                let scenery = setup_scenery(world);

                let asteroid_1 = create_asteroid(
                    world,
                    Location::Space {
                        pos: V2::new(0.0, 0.0),
                        sector_id: scenery.sector_scenery.sector_1,
                    },
                    scenery.ware_id,
                );

                let asteroid_2 = create_asteroid(
                    world,
                    Location::Space {
                        pos: V2::new(0.5, 0.0),
                        sector_id: scenery.sector_scenery.sector_1,
                    },
                    scenery.ware_id,
                );

                let asteroids = vec![scenery.asteroid, asteroid_1, asteroid_2];

                let mut miners = vec![scenery.miner];
                for _ in 1..miners_count {
                    let miner = create_miner(
                        world,
                        Location::Dock {
                            docked_id: scenery.station,
                        },
                    );

                    miners.push(miner);
                }

                // update index
                // TODO: how to test it easy without manually manipulating the index?
                let index = &mut world.write_resource::<EntityPerSectorIndex>();
                index.add_extractable(scenery.sector_scenery.sector_1, asteroid_1);
                index.add_extractable(scenery.sector_scenery.sector_1, asteroid_2);

                (scenery, asteroids, miners)
            });

            let mut targets_sector_0 = 0i32;
            let mut targets_sector_1 = 0i32;

            for miner_id in miners {
                let target_id = match world.read_storage::<Command>().get(miner_id).cloned() {
                    Some(Command::Mine(MineState { mine_target_id, .. })) => mine_target_id,
                    other => panic!("unexpected {:?} for {:?}", other, miner_id),
                };

                if target_id == Some(asteroids[0]) {
                    targets_sector_1 += 1;
                } else {
                    targets_sector_0 += 1;
                }
            }

            assert_eq!(targets_sector_0, expect_targets_sector_0 as i32);
            assert_eq!(targets_sector_1, expect_targets_sector_1 as i32);
        }

        execute(1, 0, 1);
        execute(2, 0, 2);
        execute(3, 1, 2);
    }
}
