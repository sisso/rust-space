use crate::game::commands::{Command, TradeState};
use crate::game::locations::{EntityPerSectorIndex, LocationDocked, LocationSpace, Locations};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::objects::ObjId;
use crate::game::order::TradeOrders;

use crate::game::wares::{Cargo, Cargos, WareId};
use crate::utils;
use crate::utils::{DeltaTime, TotalTime};

use rand::RngCore;
use specs::prelude::*;
use std::borrow::{Borrow, BorrowMut};

pub struct CommandTradeSystem;

#[derive(SystemData)]
pub struct CommandTradeData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    locations: ReadStorage<'a, LocationSpace>,
    locations_docked: ReadStorage<'a, LocationDocked>,
    commands: WriteStorage<'a, Command>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    orders: ReadStorage<'a, TradeOrders>,
}

impl<'a> System<'a> for CommandTradeSystem {
    type SystemData = CommandTradeData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        log::trace!("running");
        let sectors_index = &data.sector_index;

        let mut deliver_targets: Vec<ObjId> = vec![];
        let mut pickup_targets: Vec<ObjId> = vec![];

        let mut idlers_pickup = BitSet::new();
        let mut idlers_deliver = BitSet::new();
        let mut pickup_traders = BitSet::new();
        let mut deliver_traders = BitSet::new();

        let mut back_to_idle = BitSet::new();
        let mut discard_cargo = BitSet::new();

        let mut rnd = rand::thread_rng();

        let total_time = data.total_time.borrow();

        // split traders between state
        for (id, command, cargo, _) in (
            &*data.entities,
            &mut data.commands,
            &data.cargos,
            !&data.navigation,
        )
            .join()
        {
            let command: &mut Command = command;

            let trade_state = match command {
                Command::Trade(state) => state,
                _ => continue,
            };

            match trade_state {
                TradeState::Idle if cargo.is_empty() => {
                    log::trace!("{:?} selected as idle with empty cargo", id);
                    idlers_pickup.add(id.id());
                }
                TradeState::Idle => {
                    log::trace!("{:?} selected as idle", id);
                    idlers_deliver.add(id.id());
                }
                TradeState::PickUp { .. } if cargo.is_full() => {
                    log::trace!("{:?} on pick up with full cargo, update as idle", id);
                    *command = Command::Trade(TradeState::Idle);
                    idlers_deliver.add(id.id());
                }
                TradeState::PickUp { target_id, .. } => {
                    log::trace!("{:?} selected on pick up", id);
                    pickup_targets.push(*target_id);
                    pickup_traders.add(id.id());
                }
                TradeState::Deliver { .. } if cargo.is_empty() => {
                    log::trace!("{:?} on deliver with empty cargo, selected as idle", id);
                    *command = Command::Trade(TradeState::Idle);
                    idlers_pickup.add(id.id());
                }
                TradeState::Deliver { target_id, .. } => {
                    log::trace!("{:?} selected as deliver", id);
                    deliver_targets.push(*target_id);
                    deliver_traders.add(id.id());
                }
                TradeState::Delay { deadline } if total_time.is_after(*deadline) => {
                    log::trace!("{:?} delayed with deadline, selected as idle", id);
                    *command = Command::Trade(TradeState::Idle);
                    idlers_pickup.add(id.id());
                }
                TradeState::Delay { .. } => {
                    log::trace!("{:?} delayed, skipping", id);
                }
            };
        }

        let orders = &data.orders;
        let cargos = &data.cargos;

        // choose targets for pickup
        for (_, id, command) in (idlers_pickup, &*data.entities, &mut data.commands).join() {
            let sector_id =
                Locations::resolve_space_position(&data.locations, &data.locations_docked, id)
                    .unwrap()
                    .sector_id;

            let candidates = sectors_index.search_nearest_stations(sector_id).flat_map(
                |(_sector_id, distance, candidate_id)| match orders
                    .get(candidate_id)
                    .map(|orders| orders.wares_provider())
                {
                    Some(wares) if !wares.is_empty() => {
                        if let Some(station_cargo) = cargos.get(candidate_id) {
                            if wares
                                .iter()
                                .any(|ware_id| station_cargo.get_amount(*ware_id) > 0)
                            {
                                let active_traders = pickup_targets
                                    .iter()
                                    .filter(|id| **id == candidate_id)
                                    .count()
                                    as u32;

                                let luck = (rnd.next_u32() % 1000) as f32 / 1000.0f32;
                                let weight: f32 = distance as f32 + active_traders as f32 + luck;
                                Some((weight, candidate_id))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
            );

            match utils::next_lower(candidates) {
                Some(target_id) => {
                    pickup_targets.push(target_id);
                    pickup_traders.add(id.id());

                    let wares = orders.get(target_id).unwrap().wares_provider();

                    log::debug!(
                        "{:?} found station {:?} to pickup {:?}",
                        id,
                        target_id,
                        wares,
                    );

                    *command = Command::Trade(TradeState::PickUp { target_id, wares });
                }
                None => {
                    let wait_time = (rnd.next_u32() % 1000) as f32 / 1000.0;
                    let deadline = total_time.add(DeltaTime(wait_time));
                    *command = Command::Trade(TradeState::Delay { deadline });
                    log::debug!(
                        "{:?} can not find a station to pickup, setting wait time of {:?} seconds",
                        id,
                        wait_time,
                    );
                }
            }
        }

        // choose targets for deliver
        for (_, id, cargo, command) in (
            idlers_deliver,
            &*data.entities,
            &data.cargos,
            &mut data.commands,
        )
            .join()
        {
            let sector_id =
                Locations::resolve_space_position(&data.locations, &data.locations_docked, id)
                    .unwrap()
                    .sector_id;

            let wares_in_cargo: Vec<WareId> = cargo.get_wares_ids().collect();

            let candidates = sectors_index
                .search_nearest_stations(sector_id)
                .flat_map(|(_sector_id, distance, obj_id)| {
                    match orders
                        .get(obj_id)
                        .map(|orders| orders.request_any(&wares_in_cargo))
                    {
                        Some(wares) if !wares.is_empty() => {
                            let active_traders =
                                pickup_targets.iter().filter(|id| **id == obj_id).count() as u32;

                            let luck = (rnd.next_u32() % 1000) as f32 / 1000.0f32;
                            let weight: f32 = distance as f32 + active_traders as f32 + luck;
                            Some((weight, obj_id))
                        }
                        _ => None,
                    }
                })
                .filter(|(_weight, target_id)| {
                    if let Some(cargo) = cargos.get(*target_id) {
                        // check if any ware in cargo can be received by the stations
                        wares_in_cargo.iter().any(|ware_id| {
                            let amount = cargo.free_volume(*ware_id).unwrap_or(0);
                            amount > 0
                        })
                    } else {
                        false
                    }
                });

            match utils::next_lower(candidates) {
                Some(target_id) => {
                    deliver_targets.push(target_id);
                    deliver_traders.add(id.id());

                    let wares_requests = orders.get(target_id).unwrap().wares_requests();
                    assert!(!wares_requests.is_empty(), "request wares can not be empty");

                    let wares = wares_in_cargo
                        .into_iter()
                        .filter(|ware_id| wares_requests.contains(ware_id))
                        .collect::<Vec<WareId>>();

                    assert!(!wares.is_empty());

                    log::debug!(
                        "{:?} found station {:?} to deliver {:?}",
                        id,
                        target_id,
                        wares,
                    );

                    *command = Command::Trade(TradeState::Deliver { target_id, wares });
                }
                None => {
                    log::warn!(
                        "{:?} can not find a station to deliver, discarding cargo",
                        id,
                    );
                    discard_cargo.add(id.id());
                }
            }
        }

        // deliver
        for (_, id, command) in (deliver_traders, &*data.entities, &data.commands).join() {
            let (target_id, wares) = match &command {
                Command::Trade(TradeState::Deliver { target_id, wares }) => (*target_id, wares),
                _ => continue,
            };

            if Locations::is_docked_at(&data.locations_docked, id, target_id) {
                let transfer = Cargos::move_only(&mut data.cargos, id, target_id, wares);
                if transfer.moved.is_empty() {
                    log::warn!("{:?} fail to deliver wares {:?} to station {:?}, trader cargo is {:?}, station cargo is {:?}", id, wares, target_id, data.cargos.get(id), data.cargos.get(target_id));
                    back_to_idle.add(id.id());
                } else {
                    log::info!(
                        "{:?} deliver wares {:?} to station {:?}",
                        id,
                        transfer,
                        target_id,
                    );
                }
            } else {
                log::debug!(
                    "{:?} navigating to deliver wares {:?} at station {:?}",
                    id,
                    wares,
                    target_id,
                );
                data.nav_request
                    .borrow_mut()
                    .insert(id, NavRequest::MoveAndDockAt { target_id })
                    .unwrap();
            }
        }

        // pick up
        for (_, id, command) in (pickup_traders, &*data.entities, &data.commands).join() {
            let (target_id, wares) = match &command {
                Command::Trade(TradeState::PickUp { target_id, wares }) => (*target_id, wares),
                _ => continue,
            };

            if Locations::is_docked_at(&data.locations_docked, id, target_id) {
                let transfer = Cargos::move_only(&mut data.cargos, target_id, id, wares);
                if transfer.moved.is_empty() {
                    log::info!(
                        "{:?} fail to take wares {:?} from station {:?}, station cargo is {:?}",
                        id,
                        wares,
                        target_id,
                        data.cargos.get(target_id),
                    );
                    back_to_idle.add(id.id());
                } else {
                    log::info!(
                        "{:?} take wares {:?} from station {:?}",
                        id,
                        transfer,
                        target_id,
                    );
                }
            } else {
                log::debug!(
                    "{:?} navigating to pick wares {:?} at station {:?}",
                    id,
                    wares,
                    target_id,
                );
                data.nav_request
                    .borrow_mut()
                    .insert(id, NavRequest::MoveAndDockAt { target_id })
                    .unwrap();
            }
        }

        // switch back to idle
        for (entity, _, command) in (&*data.entities, back_to_idle, &mut data.commands).join() {
            log::info!("{:?} command set to trade idle", entity);
            *command = Command::trade();
        }

        for (_, entity, cargo) in (discard_cargo, &*data.entities, &mut data.cargos).join() {
            log::info!("{:?} discarding cargo", entity);
            cargo.clear();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::game::commands::Command;
    use crate::game::dock::HasDocking;
    use crate::game::locations::EntityPerSectorIndex;
    use crate::game::navigations::{Navigation, NavigationPlan};
    use crate::game::objects::ObjId;
    use crate::game::order::{TradeOrders, TRADE_ORDER_ID_FACTORY};
    use crate::game::sectors::SectorId;
    use crate::game::wares::{Cargo, Volume, WareId};
    use crate::test::test_system;
    use crate::utils::TotalTime;

    use crate::game::actions::Action;
    use commons::math::P2;
    use std::borrow::{Borrow, BorrowMut};

    const STATION_CARGO: Volume = 2000;
    const SHIP_CARGO: Volume = 500;

    #[derive(Debug)]
    struct SceneryResult {
        trader_id: ObjId,
        producer_station_id: ObjId,
        consumer_station_id: ObjId,
        /// produced and consumed
        ware0_id: WareId,
        /// produced and consumed
        ware1_id: WareId,
        /// other ware
        ware2_id: WareId,
        sector_id: SectorId,
    }

    fn add_station(world: &mut World, sector_id: SectorId, orders: TradeOrders) -> ObjId {
        world
            .create_entity()
            .with(LocationSpace {
                pos: P2::ZERO,
                sector_id,
            })
            .with(HasDocking::default())
            .with(orders)
            .with(Cargo::new(STATION_CARGO))
            .build()
    }

    fn add_trader(world: &mut World, sector_id: SectorId) -> ObjId {
        world
            .create_entity()
            .with(LocationSpace {
                pos: P2::ZERO,
                sector_id,
            })
            .with(Command::trade())
            .with(Cargo::new(SHIP_CARGO))
            .build()
    }

    fn setup_scenery(world: &mut World) -> SceneryResult {
        world.register::<HasDocking>();

        world.insert(TotalTime(0.0));

        let sector_id = world.create_entity().build();

        let ware0_id = world.create_entity().build();
        let ware1_id = world.create_entity().build();
        let ware2_id = world.create_entity().build();

        let producer_station_id = add_station(
            world,
            sector_id,
            TradeOrders::from_provided(TRADE_ORDER_ID_FACTORY, &[ware0_id, ware1_id]),
        );

        let consumer_station_id = add_station(
            world,
            sector_id,
            TradeOrders::from_requested(TRADE_ORDER_ID_FACTORY, &[ware0_id, ware1_id]),
        );

        let trader_id = add_trader(world, sector_id);

        // inject objects into the location index
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_id, producer_station_id);
        entities_per_sector.add_stations(sector_id, consumer_station_id);
        world.insert(entities_per_sector);

        // add cargo to produce stations
        add_cargo(world, producer_station_id, ware0_id, STATION_CARGO);

        let scenery = SceneryResult {
            trader_id,
            producer_station_id,
            consumer_station_id,
            ware0_id,
            ware1_id,
            ware2_id,
            sector_id,
        };

        log::debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn get_nav_request_dock_at(world: &World, ship_id: ObjId) -> ObjId {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            Some(NavRequest::MoveAndDockAt { target_id }) => return *target_id,

            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn assert_nav_request_dock_at(world: &World, ship_id: ObjId, expected_target_id: ObjId) {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            Some(NavRequest::MoveAndDockAt { target_id }) if *target_id == expected_target_id => {
                return
            }

            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn assert_no_nav_request(world: &World, ship_id: ObjId) {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            None => return,
            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn assert_command_trade_idle(world: &World, id: ObjId) {
        match world.read_storage::<Command>().borrow().get(id) {
            Some(Command::Trade(TradeState::Idle)) => {}
            other => {
                panic!("expected trade idle but found {:?} for {:?}", other, id);
            }
        }
    }

    fn assert_command_trade_delay(world: &World, id: ObjId) {
        match world.read_storage::<Command>().borrow().get(id) {
            Some(Command::Trade(TradeState::Delay { .. })) => {}
            other => {
                panic!("expected trade idle but found {:?} for {:?}", other, id);
            }
        }
    }

    fn set_docked_at(world: &mut World, ship_id: ObjId, target_id: ObjId) {
        world
            .write_storage::<LocationDocked>()
            .borrow_mut()
            .insert(
                ship_id,
                LocationDocked {
                    parent_id: target_id,
                },
            )
            .unwrap();
    }

    fn add_cargo(world: &mut World, obj_id: ObjId, ware_id: WareId, amount: Volume) {
        let cargo_storage = &mut world.write_storage::<Cargo>();
        let cargo = cargo_storage.get_mut(obj_id).unwrap();
        cargo.add(ware_id, amount).unwrap();
    }

    fn clear_cargo(world: &mut World, obj_id: ObjId) {
        let cargo_storage = &mut world.write_storage::<Cargo>();
        let cargo = cargo_storage.get_mut(obj_id).unwrap();
        cargo.clear();
    }

    fn assert_cargo(world: &World, obj_id: ObjId, ware_id: WareId, expected_amount: Volume) {
        let cargo_storage = &world.read_storage::<Cargo>();
        match cargo_storage
            .get(obj_id)
            .map(|cargo| cargo.get_amount(ware_id))
        {
            Some(amount) if amount == expected_amount => return,
            other => panic!("expected {:?} but found {:?}", expected_amount, other),
        };
    }

    fn set_active_navigation(world: &mut World, ship_id: ObjId) {
        world
            .write_storage::<Navigation>()
            .borrow_mut()
            .insert(
                ship_id,
                Navigation {
                    request: NavRequest::MoveToTarget { target_id: ship_id },
                    plan: NavigationPlan {
                        path: [Action::MoveToTargetPos {
                            target_id: ship_id,
                            last_position: None,
                        }]
                        .into(),
                    },
                },
            )
            .unwrap();
    }

    fn get_active_command(world: &World, ship_id: ObjId) -> Option<Command> {
        world
            .write_storage::<Command>()
            .borrow()
            .get(ship_id)
            .cloned()
    }

    fn set_active_command(world: &mut World, ship_id: ObjId, command: Command) {
        world
            .write_storage::<Command>()
            .borrow_mut()
            .insert(ship_id, command)
            .unwrap();
    }

    #[test]
    fn command_trade_when_empty_should_move_to_pickup_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            scenery
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.producer_station_id);
    }

    #[test]
    fn command_trade_when_empty_and_idle_and_station_is_empty_should_become_delay() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            clear_cargo(world, scenery.producer_station_id);
            scenery
        });

        assert_command_trade_delay(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_with_pickup_stat_at_target_and_station_is_empty_should_back_to_idle(
    ) {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            clear_cargo(world, scenery.producer_station_id);
            set_docked_at(world, scenery.trader_id, scenery.producer_station_id);

            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::PickUp {
                    target_id: scenery.producer_station_id,
                    wares: vec![scenery.ware0_id],
                }),
            );

            scenery
        });

        assert_command_trade_idle(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_and_navigation_should_keep_moving() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            set_active_navigation(world, scenery.trader_id);
            scenery
        });

        assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_in_delay_should_not_pick_new_target() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::Delay {
                    deadline: TotalTime(1.0),
                }),
            );
            scenery
        });

        assert_command_trade_delay(&world, scenery.trader_id);
        assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_and_delay_is_complete_should_not_pick_new_target() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            world.insert(TotalTime(1.1));
            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::Delay {
                    deadline: TotalTime(1.0),
                }),
            );
            scenery
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.producer_station_id);
    }

    #[test]
    fn command_trade_when_empty_and_at_target_should_pickup_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.producer_station_id);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        assert_cargo(
            &world,
            scenery.producer_station_id,
            scenery.ware0_id,
            STATION_CARGO - SHIP_CARGO,
        );
    }

    #[test]
    fn command_trade_when_pickup_should_only_take_valid_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.producer_station_id);

            clear_cargo(world, scenery.producer_station_id);
            add_cargo(world, scenery.producer_station_id, scenery.ware0_id, 10);
            add_cargo(world, scenery.producer_station_id, scenery.ware1_id, 10);
            add_cargo(world, scenery.producer_station_id, scenery.ware2_id, 10);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 10);
        assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 10);
        assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 0);
    }

    #[test]
    fn command_trade_when_full_should_move_to_deliver() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);

            scenery
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.consumer_station_id);
    }

    #[test]
    fn command_trade_when_full_with_pickup_should_move_to_deliver() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::PickUp {
                    target_id: scenery.producer_station_id,
                    wares: vec![scenery.ware0_id],
                }),
            );
            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);

            scenery
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.consumer_station_id);
    }

    #[test]
    fn command_trade_full_and_navigation_should_keep_moving() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_active_navigation(world, scenery.trader_id);

            scenery
        });

        assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_deliver_and_empty_cargo_and_at_target_should_back_to_pickup() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::Deliver {
                    target_id: scenery.consumer_station_id,
                    wares: vec![scenery.ware0_id],
                }),
            );
            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);

            scenery
        });

        let command = get_active_command(&world, scenery.trader_id).unwrap();
        match command {
            Command::Trade(TradeState::PickUp { .. }) => {}
            other => {
                panic!("expected command pickup but found {:?}", other);
            }
        }
    }

    #[test]
    fn command_trade_full_and_at_target_should_transfer_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);
            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);
            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            SHIP_CARGO,
        );
    }

    #[test]
    fn command_trade_full_and_target_has_full_cargo_should_back_to_idle() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_active_command(
                world,
                scenery.trader_id,
                Command::Trade(TradeState::Deliver {
                    target_id: scenery.consumer_station_id,
                    wares: vec![scenery.ware0_id],
                }),
            );

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);
            add_cargo(
                world,
                scenery.consumer_station_id,
                scenery.ware0_id,
                STATION_CARGO,
            );

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );
        assert_command_trade_idle(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_has_cargo_but_no_place_to_deliver_should_throw_cargo_away() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);
            add_cargo(
                world,
                scenery.consumer_station_id,
                scenery.ware0_id,
                STATION_CARGO,
            );

            scenery
        });

        assert_command_trade_idle(&world, scenery.trader_id);
        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );
    }

    #[test]
    fn command_trade_when_deliver_should_only_deliver_valid_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);
            add_cargo(world, scenery.trader_id, scenery.ware0_id, 1);
            add_cargo(world, scenery.trader_id, scenery.ware1_id, 1);
            add_cargo(world, scenery.trader_id, scenery.ware2_id, 1);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 0);
        assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 1);
    }

    #[test]
    fn command_trade_should_split_trade_between_stations() {
        let (world, (scenery, station_id_2, trader_id_2)) =
            test_system(CommandTradeSystem, |world| {
                let scenery = setup_scenery(world);

                let station_2 = add_station(
                    world,
                    scenery.sector_id,
                    TradeOrders::from_provided(TRADE_ORDER_ID_FACTORY, &[scenery.ware0_id]),
                );
                add_cargo(world, station_2, scenery.ware0_id, STATION_CARGO);

                let trader_2 = add_trader(world, scenery.sector_id);

                // TODO: remove it
                world
                    .write_resource::<EntityPerSectorIndex>()
                    .borrow_mut()
                    .add_stations(scenery.sector_id, station_2);

                (scenery, station_2, trader_2)
            });

        let mut targets = vec![
            get_nav_request_dock_at(&world, scenery.trader_id),
            get_nav_request_dock_at(&world, trader_id_2),
        ];
        targets.sort();

        let mut expected = vec![scenery.producer_station_id, station_id_2];
        expected.sort();

        assert_eq!(targets, expected);
    }
}
