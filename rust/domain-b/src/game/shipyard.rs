use bevy_ecs::prelude::*;
use rand::Rng;
use std::ops::Not;

use crate::game::new_obj::NewObj;
use crate::game::order::{TradeOrders, TRADE_ORDER_ID_SHIPYARD};
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::utils::DeltaTime;
use crate::game::wares::{Cargo, VecWareAmount};
use crate::game::work::WorkUnit;
use crate::game::{prefab, GameInitContext, RequireInitializer};

/// keep state of shipyard production in progress, when pending_work is <= zero, the prefab is
/// created
#[derive(Debug, Clone)]
struct ShipyardProduction {
    pending_work: WorkUnit,
    prefab_id: PrefabId,
}

/// Configure a shipyard what to produce
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProductionOrder {
    /// nothing should be produce
    None,
    /// manual set to build this
    Next(PrefabId),
    /// random production, will choose during next click
    Random,
    /// random select next prefab
    RandomSelected(PrefabId),
}

impl ProductionOrder {
    pub fn is_none(&self) -> bool {
        match self {
            ProductionOrder::None => true,
            _ => false,
        }
    }
}

/// shipyard are attached to stations and can building ships
#[derive(Debug, Clone, Component)]
pub struct Shipyard {
    pub production: WorkUnit,
    production_order: ProductionOrder,
    current_production: Option<ShipyardProduction>,
    dirt_trade_order: bool,
}

impl Shipyard {
    pub fn new() -> Self {
        Self {
            production: 1.0,
            production_order: ProductionOrder::None,
            current_production: None,
            dirt_trade_order: false,
        }
    }

    pub fn set_production_order(&mut self, production_order: ProductionOrder) {
        self.production_order = production_order;
        self.dirt_trade_order = true;
    }

    pub fn get_production_order(&self) -> ProductionOrder {
        self.production_order
    }

    pub fn is_producing(&self) -> bool {
        self.current_production.is_some()
    }

    pub fn get_producing(&self) -> Option<PrefabId> {
        self.current_production.as_ref().map(|i| i.prefab_id)
    }
}

impl RequireInitializer for Shipyard {
    fn init(context: &mut GameInitContext) {
        todo!()
        // context
        //     .dispatcher
        //     .add(ShipyardSystem, "shipyard_system", &[]);
    }
}

// pub struct ShipyardSystem;
//
// /// automatically produce one of available fleets at random, once all resources are in place, create
// /// a new fleet and start the process
// impl<'a> System<'a> for ShipyardSystem {
//     type SystemData = (
//         Read<'a, DeltaTime>,
//         Entities<'a>,
//         WriteStorage<'a, Cargo>,
//         WriteStorage<'a, Shipyard>,
//         WriteStorage<'a, NewObj>,
//         ReadStorage<'a, Prefab>,
//         WriteStorage<'a, TradeOrders>,
//     );
//
//     fn run(
//         &mut self,
//         (delta_time, entities, mut cargos, mut shipyards, mut new_objects, prefabs, mut orders): Self::SystemData,
//     ) {
//         log::trace!("running");
//
//         // collect all prefabs as candidates for random production
//         let prefabs_candidates: Vec<_> = (&entities, &prefabs)
//             .join()
//             .filter(|(_, p)| p.shipyard)
//             .collect();
//
//         let mut produced_fleets = vec![];
//         // let mut orders_updates = vec![];
//
//         // assert all shipyards have trade orders
//         for (id, _, _) in (&entities, &shipyards, orders.not()).join() {
//             panic!("shipyard {:?} do not have trade orders", id);
//         }
//
//         for (shipyard_id, cargo, shipyard, trade_order) in
//             (&*entities, &mut cargos, &mut shipyards, &mut orders).join()
//         {
//             if let Some(sp) = &mut shipyard.current_production {
//                 // update progress
//                 sp.pending_work -= shipyard.production * delta_time.as_f32();
//
//                 let is_complete = sp.pending_work <= 0.0;
//                 if is_complete {
//                     // move out the reference to allow us to change current production
//                     let prefab_id = sp.prefab_id;
//
//                     // complete current production
//                     shipyard.current_production = None;
//
//                     // create produced prefab
//                     if let Some(mut new_obj) = prefab::get_by_id(&prefabs, prefab_id) {
//                         // put into shipyard
//                         new_obj = new_obj.at_dock(shipyard_id);
//                         log::debug!("{:?} complete production of {:?}", shipyard_id, new_obj);
//                         produced_fleets.push(new_obj);
//                     } else {
//                         log::warn!(
//                             "{:?} fail to produce fleet, prefab id {:?} not found, ignoring production",
//                             shipyard_id,
//                             prefab_id
//                         );
//                     }
//                 } else {
//                     log::trace!(
//                         "{:?} producing, still need {:?} work",
//                         shipyard_id,
//                         sp.pending_work
//                     );
//                 }
//             } else {
//                 let (prefab_id, clean_on_build) = match shipyard.production_order {
//                     ProductionOrder::None => {
//                         log::trace!("{:?} no producing order, skipping", shipyard_id);
//                         continue;
//                     }
//                     ProductionOrder::Next(prefab_id) => (prefab_id, true),
//                     ProductionOrder::Random => {
//                         let index = rand::thread_rng().gen_range(0..prefabs_candidates.len());
//                         let (prefab_id, _) = prefabs_candidates[index];
//                         (prefab_id, false)
//                     }
//                     ProductionOrder::RandomSelected(prefab_id) => (prefab_id, false),
//                 };
//
//                 let prefab = match prefabs_candidates.iter().find(|(e, _)| *e == prefab_id) {
//                     Some((_, prefab)) => prefab,
//                     None => {
//                         log::warn!(
//                             "shipyard could not find prefab from id {:?}, skipping",
//                             prefab_id
//                         );
//                         continue;
//                     }
//                 };
//
//                 let production_cost = match prefab.obj.production_cost.as_ref() {
//                     Some(value) => value,
//                     None => {
//                         log::warn!(
//                             "prefab_id {:?} do not have production cost, skipping",
//                             prefab_id
//                         );
//                         continue;
//                     }
//                 };
//
//                 // check if have enough resources
//                 if cargo.remove_all_or_none(&production_cost.cost).is_ok() {
//                     // setup completion
//                     shipyard.current_production = Some(ShipyardProduction {
//                         pending_work: production_cost.work,
//                         prefab_id,
//                     });
//
//                     // update next order
//                     if clean_on_build {
//                         shipyard.production_order = ProductionOrder::None;
//                     } else {
//                         shipyard.production_order = ProductionOrder::Random;
//                     }
//
//                     // remove requesting orders
//                     trade_order.remove_by_id(TRADE_ORDER_ID_SHIPYARD);
//
//                     log::debug!(
//                         "{:?} staring production of prefab {:?}, expected to be complete at {:?}, next order is {:?}",
//                         shipyard_id,
//                         prefab_id,
//                         production_cost.work / shipyard.production,
//                         shipyard.production_order,
//                     );
//                 } else {
//                     log::trace!(
//                         "{:?} can not start production of {:?}, not enough resources",
//                         shipyard_id,
//                         prefab_id
//                     );
//
//                     if shipyard.dirt_trade_order {
//                         // update trade orders
//                         shipyard.dirt_trade_order = false;
//                         trade_order.remove_by_id(TRADE_ORDER_ID_SHIPYARD);
//                         let requested_wares = production_cost.cost.get_wares_id();
//                         log::trace!(
//                             "{:?} updating trading orders to request {:?} ",
//                             shipyard_id,
//                             requested_wares
//                         );
//                         for ware_id in requested_wares {
//                             trade_order.add_request(TRADE_ORDER_ID_SHIPYARD, ware_id);
//                         }
//                     }
//                 }
//             }
//         }
//
//         // create new objects
//         for obj in produced_fleets {
//             entities.build_entity().with(obj, &mut new_objects).build();
//         }
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use crate::game::code::HasCode;
//     use crate::game::commands::Command;
//     use crate::game::dock::HasDocking;
//     use crate::game::label::Label;
//     use crate::game::loader::Loader;
//     use crate::game::locations::LocationDocked;
//     use crate::game::order::TradeOrders;
//     use crate::game::wares::{Volume, WareAmount, WareId};
//     use crate::test::test_system;
//     use crate::game::utils::DeltaTime;
//
//     use super::*;
//
//     const TOTAL_WORK: f32 = 5.0;
//     const PENDING_WORK_AFTER_SECOND: f32 = TOTAL_WORK - 1.0;
//     const REQUIRE_CARGO: Volume = 50;
//     const NOT_ENOUGH_CARGO: Volume = REQUIRE_CARGO - 5;
//     const TIME_TO_WORK_COMPLETE: DeltaTime = DeltaTime(TOTAL_WORK);
//     const NOT_ENOUGH_TIME: DeltaTime = DeltaTime(1.0);
//
//     #[test]
//     fn test_shipyard_system_should_not_start_production_without_enough_cargo() {
//         let (world, (shipyard_id, ware_id, prefab_id)) =
//             scenery(NOT_ENOUGH_TIME, NOT_ENOUGH_CARGO, None, |prefab_id| {
//                 ProductionOrder::Next(prefab_id)
//             });
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, NOT_ENOUGH_CARGO);
//         assert_shipyard_production(&world, shipyard_id, None);
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::Next(prefab_id));
//         assert_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_should_not_start_production_with_enough_cargo_but_production_not_selected(
//     ) {
//         let (world, (shipyard_id, ware_id, _prefab_id)) =
//             scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |_| {
//                 ProductionOrder::None
//             });
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, REQUIRE_CARGO);
//         assert_shipyard_production(&world, shipyard_id, None);
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::None);
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_should_start_production_with_selected_order_and_current_order_changed_to_none(
//     ) {
//         let (world, (shipyard_id, ware_id, _prefab_id)) =
//             scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |prefab_id| {
//                 ProductionOrder::Next(prefab_id)
//             });
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, 0);
//         assert_shipyard_production(&world, shipyard_id, Some(TOTAL_WORK));
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::None);
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_with_random_order_should_start_production_and_keep_order_at_random() {
//         let (world, (shipyard_id, ware_id, _prefab_id)) =
//             scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |_| {
//                 ProductionOrder::Random
//             });
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, 0);
//         assert_shipyard_production(&world, shipyard_id, Some(TOTAL_WORK));
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::Random);
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_should_not_start_production_in_parallel_with_enough_cargo_but_already_producing(
//     ) {
//         let (world, (shipyard_id, ware_id, _prefab_id)) =
//             scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, Some(TOTAL_WORK), |_| {
//                 ProductionOrder::None
//             });
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, REQUIRE_CARGO);
//         assert_shipyard_production(&world, shipyard_id, Some(PENDING_WORK_AFTER_SECOND));
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_should_keep_queued_next_order_during_production() {
//         let (world, (shipyard_id, ware_id, prefab_id)) = scenery(
//             NOT_ENOUGH_TIME,
//             REQUIRE_CARGO,
//             Some(TOTAL_WORK),
//             |prefab_id| ProductionOrder::Next(prefab_id),
//         );
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, REQUIRE_CARGO);
//         assert_shipyard_production(&world, shipyard_id, Some(PENDING_WORK_AFTER_SECOND));
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::Next(prefab_id));
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     #[test]
//     fn test_shipyard_system_should_complete_production() {
//         let (world, (shipyard_id, _ware_id, _prefab_id)) =
//             scenery(TIME_TO_WORK_COMPLETE, 0, Some(TOTAL_WORK), |_| {
//                 ProductionOrder::None
//             });
//         assert_shipyard_production(&world, shipyard_id, None);
//         assert_not_buy_order(&world, shipyard_id);
//
//         let storage = &world.read_storage::<NewObj>();
//
//         let new_obj: &NewObj = storage.as_slice().iter().last().unwrap();
//
//         assert!(new_obj.speed.is_some());
//
//         match &new_obj.location_docked {
//             Some(LocationDocked { parent_id }) => {
//                 assert_eq!(*parent_id, shipyard_id);
//             }
//             other => {
//                 panic!("unexpected location {:?}", other);
//             }
//         }
//
//         // check command
//         assert!(new_obj.command.is_some());
//     }
//
//     #[test]
//     fn test_shipyard_system_on_completion_should_not_start_next_selected_order_until_next_tick() {
//         let (world, (shipyard_id, ware_id, prefab_id)) = scenery(
//             TIME_TO_WORK_COMPLETE,
//             REQUIRE_CARGO,
//             Some(TOTAL_WORK),
//             |prefab_id| ProductionOrder::Next(prefab_id),
//         );
//         assert_shipyard_production(&world, shipyard_id, None);
//         assert_shipyard_selected(&world, shipyard_id, ProductionOrder::Next(prefab_id));
//         assert_shipyard_cargo(&world, shipyard_id, ware_id, REQUIRE_CARGO);
//         assert_not_buy_order(&world, shipyard_id);
//     }
//
//     fn assert_shipyard_cargo(world: &World, entity: Entity, ware_id: WareId, expected: Volume) {
//         let current_cargo = world
//             .read_storage::<Cargo>()
//             .get(entity)
//             .unwrap()
//             .get_amount(ware_id);
//
//         assert_eq!(expected, current_cargo);
//     }
//
//     fn assert_shipyard_production(world: &World, entity: Entity, expected: Option<WorkUnit>) {
//         let current_production = world
//             .read_storage::<Shipyard>()
//             .get(entity)
//             .unwrap()
//             .current_production
//             .as_ref()
//             .map(|i| i.pending_work);
//
//         assert_eq!(expected, current_production);
//     }
//
//     fn assert_shipyard_selected(world: &World, entity: Entity, expected_selected: ProductionOrder) {
//         let current_production = world
//             .read_storage::<Shipyard>()
//             .get(entity)
//             .unwrap()
//             .production_order;
//         assert_eq!(expected_selected, current_production);
//     }
//
//     fn assert_buy_order(world: &World, shipyard_id: Entity) {
//         let orders = world
//             .read_storage::<TradeOrders>()
//             .get(shipyard_id)
//             .expect("orders not found for shipyard")
//             .clone();
//         assert!(orders.is_requesting());
//         assert!(!orders.is_provide());
//     }
//
//     fn assert_not_buy_order(world: &World, shipyard_id: Entity) {
//         let trade_orders = world
//             .read_storage::<TradeOrders>()
//             .get(shipyard_id)
//             .expect("fail to find shipyard trade orders")
//             .is_empty();
//         assert!(trade_orders, "trade orders are not empty");
//     }
//
//     /// returns the world and shipyard entity
//     fn scenery(
//         system_update_delta_time: DeltaTime,
//         station_current_cargo_amount: Volume,
//         current_production: Option<WorkUnit>,
//         next_order: fn(PrefabId) -> ProductionOrder,
//     ) -> (World, (Entity, WareId, PrefabId)) {
//         test_system(ShipyardSystem, move |world| {
//             world.register::<HasCode>();
//             world.register::<Label>();
//             world.register::<HasDocking>();
//
//             let ware_id = world.create_entity().with(Label::from("ore")).build();
//             let new_obj = Loader::new_ship(2.0, "fleet".to_string())
//                 .with_command(Command::mine())
//                 .with_production_cost(TOTAL_WORK, vec![WareAmount::new(ware_id, REQUIRE_CARGO)]);
//             let prefab_id = Loader::add_prefab(world, "fleet", "fleet", new_obj, true, false);
//
//             // create station prefab to check we never building it by mistake
//             Loader::add_prefab(
//                 world,
//                 "station",
//                 "station",
//                 Loader::new_station(),
//                 false,
//                 true,
//             );
//
//             assert!(station_current_cargo_amount < 1000);
//             let mut cargo = Cargo::new(1000);
//             if station_current_cargo_amount > 0 {
//                 cargo.add(ware_id, station_current_cargo_amount).unwrap();
//             }
//
//             world.insert(system_update_delta_time);
//
//             let mut shipyard = Shipyard::new();
//             shipyard.production_order = next_order(prefab_id);
//             shipyard.current_production =
//                 current_production.map(|pending_work| ShipyardProduction {
//                     pending_work,
//                     prefab_id,
//                 });
//             shipyard.dirt_trade_order = true;
//
//             let shipyard_id = world
//                 .create_entity()
//                 .with(Label::from("shipyard"))
//                 .with(cargo)
//                 .with(shipyard.clone())
//                 .with(HasDocking::default())
//                 .with(TradeOrders::default())
//                 .build();
//             log::trace!("creating shipyard {:?} {:?}", shipyard_id, shipyard);
//
//             (shipyard_id, ware_id, prefab_id)
//         })
//     }
// }
