use bevy_ecs::prelude::*;

use super::*;
use crate::game::extractables::Extractable;
use crate::game::locations::{EntityPerSectorIndex, LocationDocked, LocationOrbit, LocationSpace};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::order::TradeOrders;
use crate::game::wares::{Cargo, WareId};
use commons::unwrap_or_continue;

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
pub fn system_command_mine(
    mut commands: Commands,
    mut query: Query<
        (Entity, Option<&LocationOrbit>, &mut Command),
        (Without<Navigation>, Without<ActionExtract>),
    >,
    query_locations: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    query_extractables: Query<(Entity, &Extractable, &LocationSpace)>,
    query_orders: Query<&TradeOrders>,
    mut query_cargos: Query<&mut Cargo>,
    sector_index: Res<EntityPerSectorIndex>,
) {
    log::trace!("running");

    let mut cargo_transfers = vec![];
    let mut already_targets: HashMap<ObjId, u32> = HashMap::new();

    // collect all already target extractables to avoid funnel into same target
    for (_, _, command) in &query {
        match command {
            Command::Mine(mine) => {
                for target_id in &mine.mine_target_id {
                    *already_targets.entry(*target_id).or_insert(0) += 1;
                }
            }
            _ => {}
        };
    }

    for (id, maybe_orbit, mut command) in &mut query {
        let command = match command.as_mine_mut() {
            Some(mine) => mine,
            _ => continue,
        };

        let cargo = unwrap_or_continue!(query_cargos.get_mut(id).ok());

        if cargo.is_full() {
            // deliver cargo

            // get or search for a target if not yet defined
            let target_id = match command.deliver_target_id {
                Some(id) => id,
                None => {
                    let sector_id = Locations::resolve_space_position(&query_locations, id)
                        .unwrap()
                        .sector_id;

                    let wares_to_deliver: Vec<WareId> = cargo.get_wares_ids().collect();

                    match search_orders_target(
                        &sector_index,
                        sector_id,
                        &query_orders,
                        Some(&wares_to_deliver),
                        Vec::new(),
                        false,
                    ) {
                        Some((target_id, _wares)) => {
                            log::trace!("{:?} cargo full, setting target to {:?}", id, target_id);
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

            if Locations::is_docked_at(&query_locations, id, target_id) {
                let target_has_space = !query_cargos
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
                commands
                    .entity(id)
                    .insert(NavRequest::MoveAndDockAt { target_id });
            }
        } else {
            // cargo is not full
            let target_id = match command.mine_target_id {
                Some(id) => id,
                None => {
                    let sector_id = Locations::resolve_space_position(&query_locations, id)
                        .unwrap()
                        .sector_id;

                    let target_id =
                        match search_mine_target(&sector_index, &already_targets, sector_id) {
                            Some(target_id) => target_id,
                            None => {
                                log::debug!("{:?} fail to find any target to mine, ignoring", id);
                                continue;
                            }
                        };

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
                    let ware_id = query_extractables
                        .get(target_id)
                        .expect("target extractable not found")
                        .1
                        .ware_id;

                    log::debug!(
                        "{:?} orbiting extractable {:?}, start extraction of {:?}",
                        id,
                        target_id,
                        ware_id,
                    );

                    commands
                        .entity(id)
                        .insert(ActionRequest(Action::Extract { target_id, ware_id }));
                }
                _ => {
                    log::debug!("{:?} command to move to extractable {:?}", id, target_id,);
                    // move to target
                    commands
                        .entity(id)
                        .insert(NavRequest::OrbitTarget { target_id });
                }
            }
        }
    }

    // transfer all cargos
    for (from_id, to_id) in cargo_transfers {
        let transfer = Cargos::move_all(&mut query_cargos, from_id, to_id);
        log::info!("{:?} transfer {:?} to {:?}", from_id, transfer, to_id);
    }
}

fn search_mine_target(
    sectors_index: &EntityPerSectorIndex,
    already_targets: &HashMap<ObjId, u32>,
    sector_id: SectorId,
) -> Option<ObjId> {
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
    candidates.iter().next().map(|(_, target_id)| *target_id)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::dock::HasDocking;
    use crate::game::label::Label;
    use crate::game::loader::Loader;
    use crate::game::order::TRADE_ORDER_ID_EXTRACTABLE;
    use bevy_ecs::system::RunSystemOnce;

    use crate::game::sectors::test_scenery::SectorScenery;
    use crate::game::utils::V2;
    use crate::game::wares::{Volume, WareId};

    pub const STATION_CARGO: Volume = 1000;
    pub const MINER_CARGO: Volume = 100;

    #[derive(Debug)]
    struct SceneryResult {
        sector_scenery: SectorScenery,
        miner_id: ObjId,
        asteroid_id: ObjId,
        station_id: ObjId,
        ware_id: WareId,
    }

    fn create_asteroid(world: &mut World, location: LocationSpace, ware_id: WareId) -> ObjId {
        world
            .spawn_empty()
            .insert(location)
            .insert(Extractable {
                ware_id,
                accessibility: 1.0,
            })
            .id()
    }

    fn create_miner(world: &mut World, docked_at: ObjId) -> ObjId {
        world
            .spawn_empty()
            .insert(LocationDocked {
                parent_id: docked_at,
            })
            .insert(Command::mine())
            .insert(Cargo::new(MINER_CARGO))
            .id()
    }

    /// Setup a asteroid in sector 0, a mine station in sector 1, a miner docked in the station
    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_scenery = test_scenery::setup_sector_scenery(world);

        let ware_id = world.spawn_empty().insert(Label::from("ore")).id();

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
            .spawn_empty()
            .insert(Label::from("station"))
            .insert(LocationSpace {
                pos: V2::new(0.0, 0.0),
                sector_id: sector_scenery.sector_0,
            })
            .insert(HasDocking::default())
            .insert(Cargo::new(STATION_CARGO))
            .insert(orders)
            .id();

        let miner_id = create_miner(world, station_id);

        // inject objects into the location index
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_scenery.sector_0, station_id);
        entities_per_sector.add_extractable(sector_scenery.sector_0, asteroid);
        world.insert_resource(entities_per_sector);

        let scenery = SceneryResult {
            sector_scenery,
            miner_id: miner_id,
            asteroid_id: asteroid,
            station_id: station_id,
            ware_id,
        };

        log::debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn remove_station_ware_order(world: &mut World, scenery: &SceneryResult) {
        world.entity_mut(scenery.station_id).remove::<TradeOrders>();
    }

    fn set_miner_to_asteroid_orbit(world: &mut World, scenery: &SceneryResult) {
        Loader::set_obj_to_obj_orbit(world, scenery.miner_id, scenery.asteroid_id);
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        world.run_system_once(system_command_mine);

        match world.get::<Command>(scenery.miner_id).unwrap().as_mine() {
            Some(command) => {
                assert_eq!(command.mine_target_id, Some(scenery.asteroid_id));
            }
            None => {
                panic!("miner has no commandmine");
            }
        }

        match world.get::<NavRequest>(scenery.miner_id) {
            Some(NavRequest::OrbitTarget { target_id: target }) => {
                assert_eq!(target.clone(), scenery.asteroid_id);
            }
            other => panic!("invalid request {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_mine() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        set_miner_to_asteroid_orbit(&mut world, &scenery);

        world.run_system_once(system_command_mine);

        match world.get::<ActionRequest>(scenery.miner_id) {
            Some(ActionRequest(Action::Extract { target_id, ware_id })) => {
                assert_eq!(*target_id, scenery.asteroid_id);
                assert_eq!(*ware_id, scenery.ware_id);
            }

            other => panic!("unexpected {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_be_wait_if_cargo_is_full_and_has_no_target_station() {
        let mut world = World::new();

        let scenery = setup_scenery(&mut world);
        remove_station_ware_order(&mut world, &scenery);
        world
            .get_mut::<Cargo>(scenery.miner_id)
            .unwrap()
            .add_to_max(scenery.ware_id, 100);

        world.run_system_once(system_command_mine);

        assert!(world.get::<NavRequest>(scenery.miner_id).is_none());
    }

    #[test]
    fn test_command_mine_should_navigate_to_station_when_cargo_is_full() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        set_miner_to_asteroid_orbit(&mut world, &scenery);
        world
            .get_mut::<Cargo>(scenery.miner_id)
            .unwrap()
            .add_to_max(scenery.ware_id, 100);

        world.run_system_once(system_command_mine);

        match world.get::<NavRequest>(scenery.miner_id) {
            Some(NavRequest::MoveAndDockAt { target_id: target }) => {
                assert_eq!(target.clone(), scenery.station_id);
            }
            other => panic!("unexpected nav request {:?}", other),
        }
    }

    #[test]
    fn test_command_mine_should_not_deliver_cargo_to_station_if_not_require() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        remove_station_ware_order(&mut world, &scenery);
        world
            .get_mut::<Cargo>(scenery.miner_id)
            .unwrap()
            .add_to_max(scenery.ware_id, 100);

        world.run_system_once(system_command_mine);

        assert_eq!(
            100,
            world
                .get::<Cargo>(scenery.miner_id)
                .unwrap()
                .get_current_volume()
        );
    }

    #[test]
    fn test_command_mine_should_deliver_cargo_to_station_when_docked() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        Loader::set_cargo_to_max(&mut world, scenery.miner_id, scenery.ware_id);

        world.run_system_once(system_command_mine);

        Loader::assert_cargo(&mut world, scenery.miner_id, scenery.ware_id, 0);
        Loader::assert_cargo(&mut world, scenery.station_id, scenery.ware_id, MINER_CARGO);
    }

    #[test]
    fn test_command_mine_should_wait_if_target_station_is_full() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        world
            .get_mut::<Cargo>(scenery.miner_id)
            .unwrap()
            .add_to_max(scenery.ware_id, MINER_CARGO);
        world
            .get_mut::<Cargo>(scenery.station_id)
            .unwrap()
            .add_to_max(scenery.ware_id, STATION_CARGO);

        world.run_system_once(system_command_mine);

        assert_eq!(
            MINER_CARGO,
            world
                .get::<Cargo>(scenery.miner_id)
                .unwrap()
                .get_current_volume()
        );
        assert_eq!(
            STATION_CARGO,
            world
                .get::<Cargo>(scenery.station_id)
                .unwrap()
                .get_current_volume()
        );
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
            let mut world = World::new();
            let scenery = setup_scenery(&mut world);

            let asteroid_1 = create_asteroid(
                &mut world,
                LocationSpace {
                    pos: V2::new(0.0, 0.0),
                    sector_id: scenery.sector_scenery.sector_1,
                },
                scenery.ware_id,
            );

            let asteroid_2 = create_asteroid(
                &mut world,
                LocationSpace {
                    pos: V2::new(0.5, 0.0),
                    sector_id: scenery.sector_scenery.sector_1,
                },
                scenery.ware_id,
            );

            let asteroids = vec![scenery.asteroid_id, asteroid_1, asteroid_2];

            let mut miners = vec![scenery.miner_id];
            for _ in 1..miners_count {
                let miner = create_miner(&mut world, scenery.station_id);
                miners.push(miner);
            }

            // update index
            let index = &mut world.get_resource_mut::<EntityPerSectorIndex>().unwrap();
            index.add_extractable(scenery.sector_scenery.sector_1, asteroid_1);
            index.add_extractable(scenery.sector_scenery.sector_1, asteroid_2);

            world.run_system_once(system_command_mine);

            let mut targets_sector_0 = 0i32;
            let mut targets_sector_1 = 0i32;

            for miner_id in miners {
                let target_id = match world.get::<Command>(miner_id) {
                    Some(Command::Mine(MineState { mine_target_id, .. })) => *mine_target_id,
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
