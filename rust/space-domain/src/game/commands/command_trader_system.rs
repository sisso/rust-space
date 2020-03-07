use specs::prelude::*;
use crate::game::locations::{Location, EntityPerSectorIndex, Locations};
use crate::game::commands::Command;
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::wares::{Cargo, WareId, Cargos};
use crate::game::dock::HasDock;
use crate::game::order::Orders;
use crate::game::sectors::SectorId;
use crate::game::objects::ObjId;
use std::borrow::BorrowMut;

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

// TODO: only pick up valid that can be delivered
// TODO: huge refactorying is require to remove copy paste from command mine
impl<'a> System<'a> for CommandTradeSystem {
    type SystemData = CommandTradeData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        trace!("running");

        let sectors_index = &data.sector_index;
        let locations = &data.locations;
        // TODO: only move ores that should move
        let mut cargo_transfers = vec![];

        // ships with trade command but not executing any navigation
        for (entity, command, cargo, _, location) in (
            &*data.entities,
            &mut data.commands,
            &data.cargos,
            !&data.navigation,
            &data.locations,
        )
            .join()
        {
            // re-assign to stop Intelij to complain
            let command: &mut Command = command;

            let state = match command {
                Command::Trade(trade) => trade,
                _ => continue,
            };

            if cargo.is_empty() {
                let target_id = match state.pickup_target_id {
                    Some(target_id) => target_id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        match search_pickup_target(sectors_index, entity, sector_id, &data.orders) {
                            Some(target_id) => {
                                debug!("{:?} found station {:?} to pickup", entity, target_id);
                                state.pickup_target_id = Some(target_id);
                                target_id
                            },
                            None => {
                                warn!("{:?} can not find a cargo for pickup", entity);
                                continue;
                            }
                        }
                    }
                };

                if location.as_docked() == Some(target_id) {
                    debug!("{:?} take wares from station {:?}", entity, target_id);
                    cargo_transfers.push((target_id, entity));
                } else {
                    debug!("{:?} navigating to pick wares at station {:?}", entity, target_id);
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveAndDockAt { target_id })
                        .unwrap();
                }
            } else {
                let target_id = match state.deliver_target_id {
                    Some(target_id) => target_id,
                    None => {
                        let sector_id = Locations::resolve_space_position(locations, entity)
                            .unwrap()
                            .sector_id;

                        let wares = cargo.get_wares().cloned().collect();

                        match super::search_deliver_target(sectors_index, entity, sector_id, &data.orders, &wares) {
                            Some(target_id) => {
                                debug!("{:?} found station {:?} to deliver {:?}", entity, target_id, wares);
                                state.deliver_target_id = Some(target_id);
                                target_id
                            },
                            None => {
                                warn!("{:?} can not find a station to deliver", entity);
                                continue;
                            }
                        }
                    }
                };

                if location.as_docked() == Some(target_id) {
                    debug!("{:?} deliver wares to station {:?}", entity, target_id);
                    cargo_transfers.push((entity, target_id));
                } else {
                    debug!("{:?} navigating to deliver wares to station {:?}", entity, target_id);
                    data.nav_request
                        .borrow_mut()
                        .insert(entity, NavRequest::MoveAndDockAt { target_id })
                        .unwrap();
                }
            }
        }

        // transfer all cargo
        let cargos = data.cargos.borrow_mut();
        for (from_id, to_id) in cargo_transfers {
            let transfer = Cargos::move_all(cargos, from_id, to_id);
            info!("{:?} transfer {:?} to {:?}", from_id, transfer, to_id);
        }
    }
}

fn search_pickup_target(
    sectors_index: &EntityPerSectorIndex,
    _entity: Entity,
    sector_id: SectorId,
    orders: &ReadStorage<Orders>,
) -> Option<ObjId> {
    // find nearest deliver
    let candidates = sectors_index.search_nearest_stations(sector_id);

    candidates.iter()
        .flat_map(|(sector_id, candidate_id)| {
            let has_request =
                orders.get(*candidate_id)
                    .map(|orders| !orders.wares_provider().is_empty())
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
    use crate::utils::V2_ZERO;

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
    }

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let sector_id = world.create_entity().build();

        let ware0_id = world.create_entity().build();
        let ware1_id = world.create_entity().build();

        let producer_station_id = world
            .create_entity()
            .with(Location::Space { pos: V2_ZERO, sector_id })
            .with(HasDock)
            .with(Orders::new(Order::WareProvide { wares_id: vec![ware0_id] }))
            .with(Cargo::new(STATION_CARGO))
            .build();

        let consumer_station_id = world
            .create_entity()
            .with(Location::Space { pos: V2_ZERO, sector_id })
            .with(HasDock)
            .with(Orders::new(Order::WareRequest { wares_id: vec![ware0_id, ware1_id] }))
            .with(Cargo::new(STATION_CARGO))
            .build();

        let trader_id = world
            .create_entity()
            .with(Location::Space { pos: V2_ZERO, sector_id })
            .with(Command::trade())
            .with(Cargo::new(SHIP_CARGO))
            .build();

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
        };

        debug!("setup scenery {:?}", scenery);

        scenery
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
}
