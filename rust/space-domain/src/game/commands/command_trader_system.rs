use specs::prelude::*;
use crate::game::locations::{Location, EntityPerSectorIndex, Locations, SectorDistanceIndex};
use crate::game::commands::{Command, TradeState, Commands, MineState};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::wares::{Cargo, WareId, Cargos};
use crate::game::dock::HasDock;
use crate::game::order::{Orders, Order};
use crate::game::sectors::SectorId;
use crate::game::objects::ObjId;
use std::borrow::{BorrowMut, Borrow};
use std::process::id;
use crate::utils;

pub struct CommandTradeSystem;

#[derive(SystemData)]
pub struct CommandTradeData<'a> {
    entities: Entities<'a>,
    locations: ReadStorage<'a, Location>,
    commands: WriteStorage<'a, Command>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    docks: ReadStorage<'a, HasDock>,
    orders: ReadStorage<'a, Orders>,
}

impl<'a> System<'a> for CommandTradeSystem {
    type SystemData = CommandTradeData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        trace!("running");
        let sectors_index = &data.sector_index;

        let mut deliver_targets: Vec<ObjId> = vec![];
        let mut pickup_targets: Vec<ObjId> = vec![];

        let mut idlers_pickup = BitSet::new();
        let mut idlers_deliver = BitSet::new();
        let mut pickup_traders= BitSet::new();
        let mut deliver_traders= BitSet::new();

        // split traders between state
        for (entity, command, cargo, _) in
            (&*data.entities, &data.commands, &data.cargos, !&data.navigation).join()
        {
            match command {
                Command::Trade(TradeState::Idle) if cargo.is_empty() => {
                    idlers_pickup.add(entity.id());
                },
                Command::Trade(TradeState::Idle) => {
                    idlers_deliver.add(entity.id());
                },
                Command::Trade(TradeState::PickUp { target_id, .. }) => {
                    pickup_targets.push(*target_id);
                    pickup_traders.add(entity.id());
                },
                Command::Trade(TradeState::Deliver { target_id, .. }) => {
                    deliver_targets.push(*target_id);
                    deliver_traders.add(entity.id());
                },
                _ => continue,
            };
        }

        let orders = &data.orders;

        // choose targets for pickup
        for (_, entity, location, command) in (
            idlers_pickup,
            &*data.entities,
            &data.locations,
            &mut data.commands,
        ).join() {
            let sector_id =
                Locations::resolve_space_position_from(&data.locations, &location)
                    .unwrap()
                    .sector_id;

            let candidates =
                sectors_index.search_nearest_stations(sector_id)
                    .flat_map(|(sector_id, distance, obj_id)| {
                        match orders.get(obj_id).map(|orders| orders.is_provide()) {
                            Some(true) => {
                                let active_traders= pickup_targets.iter()
                                    .filter(|id| **id == obj_id)
                                    .count() as u32;

                                let weight = distance + active_traders;
                                Some((weight, obj_id))
                            },
                            _ => { None }
                        }
                    });

            match utils::next_lower(candidates) {
                Some(target_id) => {
                    pickup_targets.push(target_id);
                    pickup_traders.add(entity.id());

                    let wares = orders.get(target_id).unwrap().wares_provider();

                    debug!("{:?} found station {:?} to pickup {:?}", entity, target_id, wares);

                    *command = Command::Trade(TradeState::PickUp {
                        target_id,
                        wares,
                    });
                },
                None => {
                    warn!("{:?} can not find a station to pickup", entity);
                }
            }
        }

        // choose targets for deliver
        for (_, entity, location, cargo, command) in (
            idlers_deliver,
            &*data.entities,
            &data.locations,
            &data.cargos,
            &mut data.commands,
        ).join() {
            let sector_id =
                Locations::resolve_space_position_from(&data.locations, &location)
                    .unwrap()
                    .sector_id;

            let mut wares_in_cargo: Vec<WareId> = cargo.get_wares().collect();

            let candidates =
                sectors_index.search_nearest_stations(sector_id)
                    .flat_map(|(sector_id, distance, obj_id)| {
                        match orders.get(obj_id).map(|orders| orders.request_any(&wares_in_cargo)) {
                            Some(wares) if !wares.is_empty() => {
                                let active_traders= pickup_targets.iter()
                                    .filter(|id| **id == obj_id)
                                    .count() as u32;

                                let weight = distance + active_traders;
                                Some((weight, obj_id))
                            },
                            _ => { None }
                        }
                    });

            match utils::next_lower(candidates) {
                Some(target_id) => {
                    deliver_targets.push(target_id);
                    deliver_traders.add(entity.id());

                    let wares_requests = orders.get(target_id).unwrap().wares_requests();
                    assert!(!wares_requests.is_empty(), "request wares can not be empty");
                    let wares = wares_in_cargo.into_iter()
                        .filter(|ware_id| wares_requests.contains(ware_id))
                        .collect::<Vec<WareId>>();

                    assert!(!wares.is_empty());

                    debug!("{:?} found station {:?} to deliver {:?}", entity, target_id, wares);

                    *command = Command::Trade(TradeState::Deliver {
                        target_id,
                        wares,
                    });
                },
                None => {
                    warn!("{:?} can not find a station to deliver", entity);
                }
            }
        }

        // deliver
        for (_, entity, command, location) in (
            deliver_traders,
            &*data.entities,
            &data.commands,
            &data.locations,
        ).join()
        {
            let (target_id, wares) = match &command {
                Command::Trade(TradeState::Deliver{ target_id, wares}) => {
                    (*target_id, wares)
                },
                _ => continue,
            };

            if location.as_docked() == Some(target_id) {
                let transfer = Cargos::move_only(&mut data.cargos, entity, target_id, wares);
                info!("{:?} deliver wares {:?} to station {:?}", entity, transfer, target_id);
            } else {
                debug!("{:?} navigating to deliver wares {:?} at station {:?}", entity, wares, target_id);
                data.nav_request
                    .borrow_mut()
                    .insert(entity, NavRequest::MoveAndDockAt { target_id })
                    .unwrap();
            }
        }

        // pick up
        for (_, entity, command, location) in (
            pickup_traders,
            &*data.entities,
            &data.commands,
            &data.locations,
        ).join()
        {
            let (target_id, wares) = match &command {
                Command::Trade(TradeState::PickUp { target_id, wares}) => {
                    (*target_id, wares)
                },
                _ => continue,
            };

            if location.as_docked() == Some(target_id) {
                let transfer = Cargos::move_only(&mut data.cargos, target_id, entity, wares);
                info!("{:?} take wares {:?} from station {:?}", entity, transfer, target_id);
            } else {
                debug!("{:?} navigating to pick wares {:?} at station {:?}", entity, wares, target_id);
                data.nav_request
                    .borrow_mut()
                    .insert(entity, NavRequest::MoveAndDockAt { target_id })
                    .unwrap();
            }
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery;
    use crate::game::wares::{WareId, Cargo};
    use crate::test::test_system;
    use specs::DispatcherBuilder;
    use crate::game::order::{Order, Orders};
    use crate::game::objects::ObjId;
    use crate::game::dock::HasDock;
    use crate::game::commands::Command;
    use crate::game::locations::{EntityPerSectorIndex, Locations};
    use crate::game::actions::Action;
    use std::borrow::{BorrowMut, Borrow};
    use crate::game::navigations::Navigation;
    use crate::utils::{V2_ZERO, IdAsU32Support};
    use std::collections::HashSet;

    struct SceneryRequest {}

    const STATION_CARGO: f32 = 20.0;
    const SHIP_CARGO: f32 = 5.0;

    #[derive(Debug)]
    struct SceneryResult {
        trader_id: ObjId,
        producer_station_id: ObjId,
        consumer_station_id: ObjId,
        ware0_id: WareId,
        ware1_id: WareId,
        ware2_id: WareId,
        sector_id: SectorId,
    }

    fn add_station(world: &mut World, sector_id: SectorId, orders: Vec<Order>) -> ObjId {
        world
            .create_entity()
            .with(Location::Space { pos: V2_ZERO, sector_id })
            .with(HasDock)
            .with(Orders(orders))
            .with(Cargo::new(STATION_CARGO))
            .build()
    }

    fn add_trader(world: &mut World, sector_id: SectorId) -> ObjId {
        world
            .create_entity()
            .with(Location::Space { pos: V2_ZERO, sector_id })
            .with(Command::trade())
            .with(Cargo::new(SHIP_CARGO))
            .build()
    }

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_id = world.create_entity().build();

        let ware0_id = world.create_entity().build();
        let ware1_id = world.create_entity().build();
        let ware2_id = world.create_entity().build();

        let producer_station_id = add_station(world, sector_id,
            vec![Order::WareProvide { wares_id: vec![ware0_id, ware1_id] }]
        );

        let consumer_station_id = add_station(world, sector_id,
            vec![Order::WareRequest { wares_id: vec![ware0_id, ware1_id] }]
        );

        let trader_id = add_trader(world, sector_id);

        // TODO: remove it
        // inject objects into the location index
        let mut entities_per_sector = EntityPerSectorIndex::new();
        entities_per_sector.add_stations(sector_id, producer_station_id);
        entities_per_sector.add_stations(sector_id, consumer_station_id);
        world.insert(entities_per_sector);

        let scenery = SceneryResult {
            trader_id,
            producer_station_id,
            consumer_station_id,
            ware0_id,
            ware1_id,
            ware2_id,
            sector_id,
        };

        debug!("setup scenery {:?}", scenery);

        scenery
    }

    fn get_nav_request_dock_at(world: &World, ship_id: ObjId) -> ObjId  {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            Some(NavRequest::MoveAndDockAt { target_id }) =>
                return *target_id,

            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn assert_nav_request_dock_at(world: &World, ship_id: ObjId, expected_target_id: ObjId) {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            Some(NavRequest::MoveAndDockAt { target_id }) if *target_id == expected_target_id =>
                return,

            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn assert_no_nav_request(world: &World, ship_id: ObjId) {
        match world.read_storage::<NavRequest>().borrow().get(ship_id) {
            None => return,
            other => panic!("unexpected nav_request {:?}", other),
        };
    }

    fn set_docked_at(world: &mut World, ship_id: ObjId, target_id: ObjId) {
        world.write_storage::<Location>()
            .borrow_mut()
            .insert(ship_id, Location::Dock { docked_id: target_id })
            .unwrap();
    }

    fn add_cargo(world: &mut World, obj_id: ObjId, ware_id: WareId, amount: f32) {
        let cargo_storage = &mut world.write_storage::<Cargo>();
        let cargo = cargo_storage.get_mut(obj_id).unwrap();
        cargo.add(ware_id, amount).unwrap();
    }

    fn assert_cargo(world: &World, obj_id: ObjId, ware_id: WareId, expected_amount: f32) {
        let cargo_storage = &world.read_storage::<Cargo>();
        match cargo_storage.get(obj_id).map(|cargo| cargo.get_amount(ware_id)) {
            Some(amount) if amount == expected_amount => return,
            other => panic!("expected {:?} but found {:?}", expected_amount, other),
        };
    }

    fn set_active_navigation(world: &mut World, ship_id: ObjId) {
        world.write_storage::<Navigation>()
            .borrow_mut()
            .insert(ship_id, Navigation::MoveTo {})
            .unwrap();
    }

    #[test]
    fn command_trade_when_empty_should_move_to_pickup_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |world| {
            setup_scenery(world)
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.producer_station_id);
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
    fn command_trade_when_empty_and_at_target_should_pickup_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.producer_station_id);
            add_cargo(world, scenery.producer_station_id, scenery.ware0_id, STATION_CARGO);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
        assert_cargo(&world, scenery.producer_station_id, scenery.ware0_id, STATION_CARGO - SHIP_CARGO);
    }

    #[test]
    fn command_trade_when_pickup_should_only_take_valid_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.producer_station_id);
            add_cargo(world, scenery.producer_station_id, scenery.ware0_id, 1.0);
            add_cargo(world, scenery.producer_station_id, scenery.ware1_id, 1.0);
            add_cargo(world, scenery.producer_station_id, scenery.ware2_id, 1.0);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 1.0);
        assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 1.0);
        assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 0.0);
    }

    #[test]
    fn command_trade_when_with_cargo_should_move_to_deliver() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);

            scenery
        });

        assert_nav_request_dock_at(&world, scenery.trader_id, scenery.consumer_station_id);
    }

    #[test]
    fn command_trade_when_with_cargo_and_navigation_should_keep_moving() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_active_navigation(world, scenery.trader_id);

            scenery
        });

        assert_no_nav_request(&world, scenery.trader_id);
    }

    #[test]
    fn command_trade_when_with_cargo_and_at_target_should_transfer_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            add_cargo(world, scenery.trader_id, scenery.ware0_id, SHIP_CARGO);
            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0.0);
        assert_cargo(&world, scenery.consumer_station_id, scenery.ware0_id, SHIP_CARGO);
    }

    #[test]
    fn command_trade_when_deliver_should_only_deliver_valid_cargo() {
        let (world, scenery) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            set_docked_at(world, scenery.trader_id, scenery.consumer_station_id);
            add_cargo(world, scenery.trader_id, scenery.ware0_id, 1.0);
            add_cargo(world, scenery.trader_id, scenery.ware1_id, 1.0);
            add_cargo(world, scenery.trader_id, scenery.ware2_id, 1.0);

            scenery
        });

        assert_cargo(&world, scenery.trader_id, scenery.ware0_id, 0.0);
        assert_cargo(&world, scenery.trader_id, scenery.ware1_id, 0.0);
        assert_cargo(&world, scenery.trader_id, scenery.ware2_id, 1.0);
    }

    #[test]
    fn command_trade_should_split_trade_between_stations() {
        let (world, (scenery, station_id_2, trader_id_2)) = test_system(CommandTradeSystem, |mut world| {
            let scenery = setup_scenery(world);

            let station_2 = add_station(world, scenery.sector_id, vec![
                Order::WareProvide { wares_id: vec![ scenery.ware0_id ] }
            ]);

            let trader_2 = add_trader(world, scenery.sector_id);

            // TODO: remove it
            world.write_resource::<EntityPerSectorIndex>().borrow_mut()
                .add_stations(scenery.sector_id, station_2);

            (scenery, station_2, trader_2)
        });

        let mut targets = vec![
            get_nav_request_dock_at(&world, scenery.trader_id),
            get_nav_request_dock_at(&world, trader_id_2),
        ];
        targets.sort();

        let mut expected = vec![
            scenery.producer_station_id,
            station_id_2,
        ];
        expected.sort();

        assert_eq!(targets, expected);
    }
}
