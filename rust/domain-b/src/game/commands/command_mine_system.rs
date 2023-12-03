use bevy_ecs::prelude::*;

use super::*;
use crate::game::dock::HasDocking;
use crate::game::extractables::Extractable;
use crate::game::locations::{EntityPerSectorIndex, LocationDocked, LocationOrbit, LocationSpace};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::order::TradeOrders;
use crate::game::wares::{Cargo, WareId};
use std::borrow::BorrowMut;

pub struct CommandMineSystem;

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
#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    locations: ReadStorage<'a, LocationSpace>,
    docked: ReadStorage<'a, LocationDocked>,
    commands: WriteStorage<'a, Command>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    action_extract: ReadStorage<'a, ActionExtract>,
    action_request: WriteStorage<'a, ActionRequest>,
    extractable: ReadStorage<'a, Extractable>,
    _docks: ReadStorage<'a, HasDocking>,
    orders: ReadStorage<'a, TradeOrders>,
    orbiting: ReadStorage<'a, LocationOrbit>,
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
        for (id, mut command, cargo, _, _, maybe_orbit) in (
            &*data.entities,
            &mut data.commands,
            &data.cargos,
            !&data.navigation,
            !&data.action_extract,
            data.orbiting.maybe(),
        )
            .join()
        {
            let command = match &mut command {
                Command::Mine(mine) => mine,
                _ => continue,
            };

            if cargo.is_full() {
                // deliver cargo

                // get or search for a target if not yet defined
                let target_id = match command.deliver_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id =
                            Locations::resolve_space_position(locations, &data.docked, id)
                                .unwrap()
                                .sector_id;

                        let wares_to_deliver: Vec<WareId> = cargo.get_wares_ids().collect();

                        match search_orders_target(
                            sectors_index,
                            sector_id,
                            &data.orders,
                            Some(&wares_to_deliver),
                            Vec::new(),
                            false,
                        ) {
                            Some((target_id, _wares)) => {
                                log::trace!(
                                    "{:?} cargo full, setting target to {:?}",
                                    id,
                                    target_id
                                );
                                command.deliver_target_id = Some(target_id);
                                command.mine_target_id = None;
                                target_id
                            }
                            None => {
                                log::trace!(
                                    "{:?} cargo full, can not find a cargo for deliver, skipping",
                                    id
                                );
                                continue;
                            }
                        }
                    }
                };

                if Locations::is_docked_at(&data.docked, id, target_id) {
                    let target_has_space = !data
                        .cargos
                        .get(target_id)
                        .expect("target has no cargo")
                        .is_full();

                    if target_has_space {
                        log::debug!(
                            "{:?} miner cargo is full, transferring cargo to station {:?}",
                            id,
                            target_id,
                        );
                        cargo_transfers.push((id, target_id));
                    } else {
                        log::debug!(
                            "{:?} miner cargo is full, stations cargo is {:?} is full, waiting",
                            id,
                            target_id,
                        );
                    }
                } else {
                    log::debug!(
                        "{:?} cargo full, command to navigate to station {:?}",
                        id,
                        target_id,
                    );
                    data.nav_request
                        .borrow_mut()
                        .insert(id, NavRequest::MoveAndDockAt { target_id })
                        .unwrap();
                }
            } else {
                // cargo is not full
                let target_id = match command.mine_target_id {
                    Some(id) => id,
                    None => {
                        let sector_id =
                            Locations::resolve_space_position(locations, &data.docked, id)
                                .unwrap()
                                .sector_id;

                        let target_id =
                            search_mine_target(sectors_index, &already_targets, id, sector_id);
                        command.mine_target_id = Some(target_id);
                        command.deliver_target_id = None;

                        *already_targets.entry(target_id).or_insert(0) += 1;

                        log::trace!(
                            "{:?} cargo not full, target {:?} for extraction",
                            id,
                            target_id
                        );

                        target_id
                    }
                };

                // check if is already orbiting the target
                match maybe_orbit {
                    Some(orbit) if orbit.parent_id == target_id => {
                        // find extractable ware id, currently take any,
                        // future: choose based on demand
                        let ware_id = data.extractable.get(target_id).unwrap().ware_id;

                        log::debug!(
                            "{:?} orbiting extractable {:?}, start extraction of {:?}",
                            id,
                            target_id,
                            ware_id,
                        );

                        data.action_request
                            .borrow_mut()
                            .insert(id, ActionRequest(Action::Extract { target_id, ware_id }))
                            .unwrap();
                    }
                    _ => {
                        log::debug!("{:?} command to move to extractable {:?}", id, target_id,);
                        // move to target
                        data.nav_request
                            .borrow_mut()
                            .insert(id, NavRequest::OrbitTarget { target_id })
                            .unwrap();
                    }
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
    use crate::game::label::Label;
    use crate::game::loader::Loader;
    use crate::game::order::TRADE_ORDER_ID_EXTRACTABLE;

    use crate::game::sectors::test_scenery::SectorScenery;
    use crate::game::wares::WareId;
    use crate::test::{init_trace_log, test_system};

    #[derive(Debug)]
    struct SceneryResult {
        sector_scenery: SectorScenery,
        miner_id: ObjId,
        asteroid_id: ObjId,
        station: ObjId,
        ware_id: WareId,
    }

    fn create_asteroid(world: &mut World, location: LocationSpace, ware_id: WareId) -> ObjId {
        world
            .create_entity()
            .with(location)
            .with(Extractable {
                ware_id,
                accessibility: 1.0,
            })
            .build()
    }

    fn create_miner(world: &mut World, docked_at: ObjId) -> ObjId {
        world
            .create_entity()
            .with(LocationDocked {
                parent_id: docked_at,
            })
            .with(Command::mine())
            .with(Cargo::new(100))
            .build()
    }

    /// Setup a asteroid in sector 0, a mine station in sector 1, a miner docked in the station
    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_scenery = test_scenery::setup_sector_scenery(world);

        let ware_id = world.create_entity().with(Label::from("ore")).build();

        let asteroid = create_asteroid(
            world,
            LocationSpace {
                pos: V2::new(1.0, 0.0),
                sector_id: sector_scenery.sector_0,
            },
            ware_id,
        );

        let mut orders = TradeOrders::default();
        orders.add_request(TRADE_ORDER_ID_EXTRACTABLE, ware_id);

        let station_id = world
            .create_entity()
            .with(Label::from("station"))
            .with(LocationSpace {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            })
            .with(HasDocking::default())
            .with(Cargo::new(1000))
            .with(orders)
            .build();

        let miner_id = create_miner(world, station_id);

        // inject objects into the location index
        // TODO: how to test it easy?
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_scenery.sector_0, station_id);
        entities_per_sector.add_extractable(sector_scenery.sector_0, asteroid);
        world.insert(entities_per_sector);

        let scenery = SceneryResult {
            sector_scenery,
            miner_id: miner_id,
            asteroid_id: asteroid,
            station: station_id,
            ware_id,
        };

        log::debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn remove_station_ware_order(world: &mut World, scenery: &SceneryResult) {
        world
            .write_storage::<TradeOrders>()
            .borrow_mut()
            .remove(scenery.station)
            .unwrap();
    }

    fn set_miner_to_asteroid_orbit(world: &mut World, scenery: &SceneryResult) {
        Loader::set_obj_to_obj_orbit(world, scenery.miner_id, scenery.asteroid_id);
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(CommandMineSystem, |world| setup_scenery(world));

        let command_storage = world.read_component::<Command>();
        let command = command_storage
            .get(scenery.miner_id)
            .and_then(|i| i.as_mine());
        match command {
            Some(command) => {
                assert_eq!(command.mine_target_id, Some(scenery.asteroid_id));
            }
            None => {
                panic!("miner has no commandmine");
            }
        }

        let request_storage = world.read_storage::<NavRequest>();
        match request_storage.get(scenery.miner_id) {
            Some(NavRequest::OrbitTarget { target_id: target }) => {
                assert_eq!(target.clone(), scenery.asteroid_id);
            }
            other => panic!("invalid request {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_mine() {
        init_trace_log().unwrap();

        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);
            set_miner_to_asteroid_orbit(world, &scenery);
            scenery
        });
        let action = world
            .read_storage::<ActionRequest>()
            .get(scenery.miner_id)
            .cloned();
        assert!(action.is_some());

        match action.unwrap().0 {
            Action::Extract { target_id, ware_id } => {
                assert_eq!(target_id, scenery.asteroid_id);
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
            let cargo = cargo_storage.get_mut(scenery.miner_id).unwrap();
            cargo.add_to_max(scenery.ware_id, 100);

            scenery
        });

        let nav_request_storage = world.read_storage::<NavRequest>();

        match nav_request_storage.get(scenery.miner_id) {
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

            set_miner_to_asteroid_orbit(world, &scenery);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner_id).unwrap();
            cargo.add_to_max(scenery.ware_id, 100);

            scenery
        });

        let nav_request_storage = world.read_storage::<NavRequest>();

        match nav_request_storage.get(scenery.miner_id) {
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
            let cargo = cargo_storage.get_mut(scenery.miner_id).unwrap();
            cargo.add_to_max(scenery.ware_id, 100);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();
        let miner_cargo = cargo_storage.get(scenery.miner_id).unwrap();
        assert_eq!(100, miner_cargo.get_current_volume());
    }

    #[test]
    fn test_command_mine_should_deliver_cargo_to_station_when_docked() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner_id).unwrap();
            let added = cargo.add_to_max(scenery.ware_id, 100);
            assert_eq!(100, added);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();

        let miner_cargo = cargo_storage.get(scenery.miner_id).unwrap();
        assert_eq!(0, miner_cargo.get_current_volume());

        let station_cargo = cargo_storage.get(scenery.station).unwrap();
        assert_eq!(100, station_cargo.get_current_volume());
    }

    #[test]
    fn test_command_mine_should_wait_if_target_station_is_full() {
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let scenery = setup_scenery(world);

            // fill miner cargo
            let cargo_storage = &mut world.write_storage::<Cargo>();
            let cargo = cargo_storage.get_mut(scenery.miner_id).unwrap();
            cargo.add_to_max(scenery.ware_id, 100);

            // fill station to max
            let cargo = cargo_storage.get_mut(scenery.station).unwrap();
            cargo.add_to_max(scenery.ware_id, 1000);

            scenery
        });

        let cargo_storage = &world.write_storage::<Cargo>();

        let miner_cargo = cargo_storage.get(scenery.miner_id).unwrap();
        assert_eq!(100, miner_cargo.get_current_volume());

        let station_cargo = cargo_storage.get(scenery.station).unwrap();
        assert_eq!(1000, station_cargo.get_current_volume());
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
                    LocationSpace {
                        pos: V2::new(0.0, 0.0),
                        sector_id: scenery.sector_scenery.sector_1,
                    },
                    scenery.ware_id,
                );

                let asteroid_2 = create_asteroid(
                    world,
                    LocationSpace {
                        pos: V2::new(0.5, 0.0),
                        sector_id: scenery.sector_scenery.sector_1,
                    },
                    scenery.ware_id,
                );

                let asteroids = vec![scenery.asteroid_id, asteroid_1, asteroid_2];

                let mut miners = vec![scenery.miner_id];
                for _ in 1..miners_count {
                    let miner = create_miner(world, scenery.station);

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
