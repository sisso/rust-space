use specs::prelude::*;

pub struct CommandTradeSystem;

impl<'a> System<'a> for CommandTradeSystem {
    type SystemData = (Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
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
    use crate::game::locations::EntityPerSectorIndex;
    use crate::game::actions::Action;
    use std::borrow::BorrowMut;
    use crate::game::navigations::Navigation;

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
            .with(HasDock)
            .with(Orders::new(Order::WareProvide { wares_id: vec![ware0_id] }))
            .with(Cargo::new(STATION_CARGO))
            .build();

        let consumer_station_id = world
            .create_entity()
            .with(HasDock)
            .with(Orders::new(Order::WareRequest { wares_id: vec![ware0_id, ware1_id] }))
            .with(Cargo::new(STATION_CARGO))
            .build();

        let trader_id = world
            .create_entity()
            .with(Command::trade())
            .with(Cargo::new(10.0))
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

    fn assert_nav_request_dock_at(world: &World, ship_id: ObjId, target_id: ObjId) {
        unimplemented!()
    }

    fn assert_no_nav_request(world: &World, ship_id: ObjId) {
        unimplemented!()
    }

    fn set_docked_at(world: &mut World, ship_id: ObjId, target_id: ObjId) {
        unimplemented!()
    }

    fn add_cargo(world: &mut World, obj_id: ObjId, ware_id: WareId, amount: f32) {
        unimplemented!()
    }

    fn assert_cargo(world: &World, obj_id: ObjId, ware_id: WareId, amount: f32) {
        unimplemented!()
    }

    fn set_active_navigation(world: &mut World, ship_id: ObjId) {
        world.write_storage::<Navigation>()
            .borrow_mut()
            .insert(ship_id, Navigation::MoveTo {});
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
        assert_cargo(&world, scenery.producer_station_id, scenery.ware0_id, SHIP_CARGO);
    }
}
