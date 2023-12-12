use crate::game::commands::{Command, TradeState};
use crate::game::locations::{
    EntityPerSectorIndex, LocationDocked, LocationSpace, Locations, SectorDistanceIndex,
};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::objects::ObjId;
use crate::game::order::TradeOrders;

use crate::game::utils::{DeltaTime, TotalTime};
use crate::game::wares::{Cargo, Cargos, WareId};

use crate::game::utils;
use bevy_ecs::prelude::*;
use commons::unwrap_or_continue;
use rand::RngCore;
use std::borrow::{Borrow, BorrowMut};

pub fn system_command_trade(
    total_time: Res<TotalTime>,
    sectors_index: Res<EntityPerSectorIndex>,
    mut commands: Commands,
    query: Query<(Entity, &Command), Without<Navigation>>,
    query_locations: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    mut query_cargos: Query<&mut Cargo>,
    query_orders: Query<&TradeOrders>,
) {
    log::trace!("running");

    let mut deliver_targets: Vec<ObjId> = vec![];
    let mut pickup_targets: Vec<ObjId> = vec![];

    let mut idlers_pickup = vec![];
    let mut idlers_deliver = vec![];
    let mut pickup_traders = vec![];
    let mut deliver_traders = vec![];

    let mut back_to_idle = vec![];
    let mut discard_cargo = vec![];

    let mut rnd = rand::thread_rng();

    let total_time = *total_time;

    // split traders between states
    for (id, command) in &query {
        let trade_state = match command {
            Command::Trade(state) => state,
            _ => continue,
        };

        let cargo = unwrap_or_continue!(query_cargos.get(id).ok());

        match trade_state {
            TradeState::Idle if cargo.is_empty() => {
                log::trace!("{:?} selected as idle with empty cargo", id);
                idlers_pickup.push(id);
            }
            TradeState::Idle => {
                log::trace!("{:?} selected as idle", id);
                idlers_deliver.push(id);
            }
            TradeState::PickUp { .. } if cargo.is_full() => {
                log::trace!("{:?} on pick up with full cargo, update as idle", id);
                back_to_idle.push(id);
            }
            TradeState::PickUp { target_id, .. } => {
                log::trace!("{:?} selected on pick up", id);
                pickup_targets.push(*target_id);
                pickup_traders.push(id);
            }
            TradeState::Deliver { .. } if cargo.is_empty() => {
                log::trace!("{:?} on deliver with empty cargo, selected as idle", id);
                back_to_idle.push(id);
            }
            TradeState::Deliver { target_id, .. } => {
                log::trace!("{:?} selected as deliver", id);
                deliver_targets.push(*target_id);
                deliver_traders.push(id);
            }
            TradeState::Delay { deadline } if total_time.is_after(*deadline) => {
                log::trace!("{:?} delayed with deadline, selected as idle", id);
                back_to_idle.push(id);
            }
            TradeState::Delay { .. } => {
                log::trace!("{:?} delayed, skipping", id);
            }
        };
    }

    // choose targets for pickup
    for (id, _) in query.iter_many(idlers_pickup) {
        let sector_id = Locations::resolve_space_position(&query_locations, id)
            .unwrap()
            .sector_id;

        // search nearest stations that provided wares
        let candidates = sectors_index.search_nearest_stations(sector_id).flat_map(
            |(_sector_id, distance, candidate_id)| match query_orders
                .get(candidate_id)
                .ok()
                .map(|orders| orders.wares_provider())
            {
                Some(wares) if !wares.is_empty() => {
                    // check if station has cargo
                    if let Some(station_cargo) = query_cargos.get(candidate_id).ok() {
                        if wares
                            .iter()
                            .any(|ware_id| station_cargo.get_amount(*ware_id) > 0)
                        {
                            // check number of active trades already doing this route
                            let count_active_delivers = pickup_targets
                                .iter()
                                .filter(|id| **id == candidate_id)
                                .count()
                                as u32;

                            // wait based on random + distance + num of active delivers
                            let luck = (rnd.next_u32() % 1000) as f32 / 1000.0f32;
                            let weight: f32 = distance as f32 + count_active_delivers as f32 + luck;
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

        // take best candidate
        match utils::next_lower(candidates) {
            Some(target_id) => {
                pickup_targets.push(target_id);

                let wares = query_orders.get(target_id).unwrap().wares_provider();

                log::debug!(
                    "{:?} found station {:?} to pickup {:?}",
                    id,
                    target_id,
                    wares,
                );

                commands
                    .entity(id)
                    .insert(Command::Trade(TradeState::PickUp { target_id, wares }));
            }
            None => {
                let wait_time = (rnd.next_u32() % 1000) as f32 / 1000.0;
                let deadline = total_time.add(DeltaTime(wait_time));
                commands
                    .entity(id)
                    .insert(Command::Trade(TradeState::Delay { deadline }));
                log::debug!(
                    "{:?} can not find a station to pickup, setting wait time of {:?} seconds",
                    id,
                    wait_time,
                );
            }
        }
    }

    // choose targets for deliver
    for (id, _) in query.iter_many(idlers_deliver) {
        let sector_id = Locations::resolve_space_position(&query_locations, id)
            .unwrap()
            .sector_id;

        let wares_in_cargo: Vec<WareId> = unwrap_or_continue!(query_cargos.get(id).ok())
            .get_wares_ids()
            .collect();

        // search nearest candidates that accept cargo
        let candidates = sectors_index
            .search_nearest_stations(sector_id)
            .flat_map(|(_sector_id, distance, obj_id)| {
                match query_orders
                    .get(obj_id)
                    .ok()
                    .map(|orders| orders.request_any(&wares_in_cargo))
                {
                    Some(wares) if !wares.is_empty() => {
                        // weight deliver by random + distance + num active delivers
                        let count_active_traders =
                            pickup_targets.iter().filter(|id| **id == obj_id).count() as u32;

                        let luck = (rnd.next_u32() % 1000) as f32 / 1000.0f32;
                        let weight: f32 = distance as f32 + count_active_traders as f32 + luck;
                        Some((weight, obj_id))
                    }
                    _ => None,
                }
            })
            .filter(|(_weight, target_id)| {
                if let Some(cargo) = query_cargos.get(*target_id).ok() {
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

                let wares_requests = query_orders.get(target_id).unwrap().wares_requests();
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

                commands
                    .entity(id)
                    .insert(Command::Trade(TradeState::Deliver { target_id, wares }));
            }
            None => {
                log::warn!(
                    "{:?} can not find a station to deliver, discarding cargo",
                    id,
                );
                discard_cargo.push(id);
            }
        }
    }

    // deliver
    for (id, command) in query.iter_many(deliver_traders) {
        let (target_id, wares) = match &command {
            Command::Trade(TradeState::Deliver { target_id, wares }) => (*target_id, wares),
            _ => continue,
        };

        if Locations::is_docked_at(&query_locations, id, target_id) {
            let transfer = Cargos::move_only(&mut query_cargos, id, target_id, wares);
            if transfer.moved.is_empty() {
                log::warn!("{:?} fail to deliver wares {:?} to station {:?}, trader cargo is {:?}, station cargo is {:?}", id, wares, target_id, query_cargos.get(id), 
                    query_cargos.get(target_id));
                back_to_idle.push(id);
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
            commands
                .entity(id)
                .insert(NavRequest::MoveAndDockAt { target_id });
        }
    }

    // pick up
    for (id, command) in query.iter_many(pickup_traders) {
        let (target_id, wares) = match &command {
            Command::Trade(TradeState::PickUp { target_id, wares }) => (*target_id, wares),
            _ => continue,
        };

        if Locations::is_docked_at(&query_locations, id, target_id) {
            let transfer = Cargos::move_only(&mut query_cargos, target_id, id, wares);
            if transfer.moved.is_empty() {
                log::info!(
                    "{:?} fail to take wares {:?} from station {:?}, station cargo is {:?}",
                    id,
                    wares,
                    target_id,
                    query_cargos.get(target_id),
                );
                back_to_idle.push(id);
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
            commands
                .entity(id)
                .insert(NavRequest::MoveAndDockAt { target_id });
        }
    }

    // switch back to idle
    for obj_id in back_to_idle {
        log::info!("{:?} command set to trade idle", obj_id);
        commands.entity(obj_id).insert(Command::trade());
    }

    for obj_id in discard_cargo {
        log::info!("{:?} discarding cargo", obj_id);
        if let Some(mut cargo) = query_cargos.get_mut(obj_id).ok() {
            cargo.clear()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::game::commands::Command;
    use crate::game::dock::HasDocking;
    use crate::game::locations::EntityPerSectorIndex;
    use crate::game::objects::ObjId;
    use crate::game::order::{TradeOrders, TRADE_ORDER_ID_FACTORY};
    use crate::game::sectors::SectorId;
    use crate::game::utils::TotalTime;
    use crate::game::wares::{Cargo, Volume, WareId};

    use crate::game::actions::Action;
    use crate::game::loader::Loader;
    use bevy_ecs::system::RunSystemOnce;
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
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::ZERO,
                sector_id,
            })
            .insert(HasDocking::default())
            .insert(orders)
            .insert(Cargo::new(STATION_CARGO))
            .id()
    }

    fn add_trader(world: &mut World, sector_id: SectorId) -> ObjId {
        world
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::ZERO,
                sector_id,
            })
            .insert(Command::trade())
            .insert(Cargo::new(SHIP_CARGO))
            .id()
    }

    fn setup_scenery(world: &mut World) -> SceneryResult {
        world.insert_resource(TotalTime(0.0));

        let sector_id = world.spawn_empty().id();

        let ware0_id = world.spawn_empty().id();
        let ware1_id = world.spawn_empty().id();
        let ware2_id = world.spawn_empty().id();

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
        world.insert_resource(entities_per_sector);

        // add cargo to produce stations
        Loader::add_cargo(world, producer_station_id, ware0_id, STATION_CARGO);

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

    #[test]
    fn command_trade_when_empty_should_move_to_pickup_cargo() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        Loader::assert_nav_request_dock_at(&world, scenery.trader_id, scenery.producer_station_id);
    }

    #[test]
    fn command_trade_when_empty_and_idle_and_station_is_empty_should_become_delay() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        Loader::clear_cargo(&mut world, scenery.producer_station_id);

        world.run_system_once(system_command_trade);

        Loader::assert_command_trade_delay(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_with_pickup_stat_at_target_and_station_is_empty_should_back_to_idle(
    ) {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::clear_cargo(&mut world, scenery.producer_station_id);
        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.producer_station_id);
        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::PickUp {
                target_id: scenery.producer_station_id,
                wares: vec![scenery.ware0_id],
            }),
        );

        world.run_system_once(system_command_trade);

        Loader::assert_command_trade_idle(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_and_navigation_should_keep_moving() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_active_navigation(&mut world, scenery.trader_id);

        world.run_system_once(system_command_trade);

        Loader::assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_in_delay_should_not_pick_new_target() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::Delay {
                deadline: TotalTime(1.0),
            }),
        );

        world.run_system_once(system_command_trade);

        Loader::assert_command_trade_delay(&world, scenery.trader_id);
        Loader::assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_empty_and_delay_is_complete_should_pick_new_target() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        world.insert_resource(TotalTime(1.1));
        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::Delay {
                deadline: TotalTime(1.0),
            }),
        );

        // back to idle
        world.run_system_once(system_command_trade);
        // choose target
        world.run_system_once(system_command_trade);
        // create nav request
        world.run_system_once(system_command_trade);

        Loader::assert_nav_request_dock_at(&world, scenery.trader_id, scenery.producer_station_id);
    }

    #[test]
    fn command_trade_when_empty_and_at_target_should_pickup_cargo() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.producer_station_id);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::assert_cargo(
            &world,
            scenery.producer_station_id,
            scenery.ware0_id,
            STATION_CARGO - SHIP_CARGO,
        );
    }

    #[test]
    fn command_trade_when_pickup_should_only_take_valid_cargo() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.producer_station_id);
        Loader::clear_cargo(&mut world, scenery.producer_station_id);
        Loader::add_cargo(
            &mut world,
            scenery.producer_station_id,
            scenery.ware0_id,
            10,
        );
        Loader::add_cargo(
            &mut world,
            scenery.producer_station_id,
            scenery.ware1_id,
            10,
        );
        Loader::add_cargo(
            &mut world,
            scenery.producer_station_id,
            scenery.ware2_id,
            10,
        );

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 10);
        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 10);
        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 0);
    }

    #[test]
    fn command_trade_when_full_should_move_to_deliver() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);

        // back to idle
        world.run_system_once(system_command_trade);
        // create navigation
        world.run_system_once(system_command_trade);

        Loader::assert_nav_request_dock_at(&world, scenery.trader_id, scenery.consumer_station_id);
    }

    #[test]
    fn command_trade_when_full_with_pickup_should_move_to_deliver() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::PickUp {
                target_id: scenery.producer_station_id,
                wares: vec![scenery.ware0_id],
            }),
        );
        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);

        // back to idle
        world.run_system_once(system_command_trade);
        // choose target
        world.run_system_once(system_command_trade);
        // create navigation
        world.run_system_once(system_command_trade);

        Loader::assert_nav_request_dock_at(&world, scenery.trader_id, scenery.consumer_station_id);
    }

    #[test]
    fn command_trade_full_and_navigation_should_keep_moving() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::set_active_navigation(&mut world, scenery.trader_id);

        world.run_system_once(system_command_trade);

        Loader::assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_deliver_and_empty_cargo_and_at_target_should_back_to_pickup() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::Deliver {
                target_id: scenery.consumer_station_id,
                wares: vec![scenery.ware0_id],
            }),
        );
        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.consumer_station_id);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        let command = Loader::get_active_command(&world, scenery.trader_id).unwrap();
        match command {
            Command::Trade(TradeState::PickUp { .. }) => {}
            other => {
                panic!("expected command pickup but found {:?}", other);
            }
        }
    }

    #[test]
    fn command_trade_full_and_at_target_should_transfer_cargo() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);
        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.consumer_station_id);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        Loader::assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            SHIP_CARGO,
        );
    }

    #[test]
    fn command_trade_full_and_target_has_full_cargo_should_back_to_idle() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_active_command(
            &mut world,
            scenery.trader_id,
            Command::Trade(TradeState::Deliver {
                target_id: scenery.consumer_station_id,
                wares: vec![scenery.ware0_id],
            }),
        );

        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.consumer_station_id);
        Loader::add_cargo(
            &mut world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );

        world.run_system_once(system_command_trade);

        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );
        Loader::assert_command_trade_idle(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_has_cargo_but_no_place_to_deliver_should_throw_cargo_away() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.consumer_station_id);
        Loader::add_cargo(
            &mut world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );

        world.run_system_once(system_command_trade);

        Loader::assert_command_trade_idle(&world, scenery.trader_id);
        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        Loader::assert_cargo(
            &world,
            scenery.consumer_station_id,
            scenery.ware0_id,
            STATION_CARGO,
        );
    }

    #[test]
    fn command_trade_when_deliver_should_only_deliver_valid_cargo() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        Loader::set_docked_at(&mut world, scenery.trader_id, scenery.consumer_station_id);
        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware0_id, 1);
        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware1_id, 1);
        Loader::add_cargo(&mut world, scenery.trader_id, scenery.ware2_id, 1);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0);
        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 0);
        Loader::assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 1);
    }

    #[test]
    fn command_trade_should_split_trade_between_stations() {
        let mut world = World::new();
        let scenery = setup_scenery(&mut world);

        let station_id_2 = add_station(
            &mut world,
            scenery.sector_id,
            TradeOrders::from_provided(TRADE_ORDER_ID_FACTORY, &[scenery.ware0_id]),
        );
        Loader::add_cargo(&mut world, station_id_2, scenery.ware0_id, STATION_CARGO);

        let trader_id_2 = add_trader(&mut world, scenery.sector_id);

        world
            .resource_mut::<EntityPerSectorIndex>()
            .add_stations(scenery.sector_id, station_id_2);

        world.run_system_once(system_command_trade);
        world.run_system_once(system_command_trade);

        let mut targets = vec![
            Loader::get_nav_request_dock_at(&world, scenery.trader_id),
            Loader::get_nav_request_dock_at(&world, trader_id_2),
        ];
        targets.sort();

        let mut expected = vec![scenery.producer_station_id, station_id_2];
        expected.sort();

        assert_eq!(targets, expected);
    }
}
