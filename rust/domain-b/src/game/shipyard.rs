use crate::game::loader::Loader;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::ops::Not;

use crate::game::new_obj::NewObj;
use crate::game::order::{TradeOrders, TRADE_ORDER_ID_SHIPYARD};
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::utils::DeltaTime;
use crate::game::wares::{Cargo, VecWareAmount};
use crate::game::work::WorkUnit;

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

    pub fn update_production(&mut self, delta_time: DeltaTime) -> ProductionResult {
        if self.current_production.is_none() {
            return ProductionResult::NotProducing;
        }
        let current = self.current_production.as_mut().unwrap();
        current.pending_work -= self.production * delta_time.as_f32();
        if current.pending_work <= 0.0 {
            let prefab_id = current.prefab_id;
            self.current_production = None;
            ProductionResult::Completed(prefab_id)
        } else {
            ProductionResult::Producing
        }
    }
}

enum ProductionResult {
    NotProducing,
    Completed(PrefabId),
    Producing,
}

/// automatically produce one of available fleets at random, once all resources are in place, create
/// a new fleet and start the process
fn system_shipyard(
    mut commands: Commands,
    delta_time: Res<DeltaTime>,
    query_prefabs: Query<(Entity, &Prefab)>,
    mut query: Query<(Entity, &mut Shipyard, Option<&mut TradeOrders>, &mut Cargo)>,
) {
    log::trace!("running");

    let delta_time = *delta_time;

    // collect all prefabs as candidates for random production
    let prefabs_candidates: Vec<_> = query_prefabs.iter().filter(|(_, p)| p.shipyard).collect();

    for (shipyard_id, mut shipyard, mut trade_order, mut cargo) in &mut query {
        let mut trade_order = match trade_order {
            Some(to) => to,
            None => {
                log::warn!("{:?} has no trade orders", shipyard_id);
                continue;
            }
        };

        match shipyard.update_production(delta_time) {
            ProductionResult::Completed(prefab_id) => {
                // move out the reference to allow us to change current production
                // complete current production
                shipyard.current_production = None;

                // create produced prefab
                if let Some((_, prefab)) =
                    prefabs_candidates.iter().find(|(id, _)| *id == prefab_id)
                {
                    let mut new_obj = prefab.obj.clone();

                    // put into shipyard
                    new_obj = new_obj.at_dock(shipyard_id);
                    log::debug!("{:?} complete production of {:?}", shipyard_id, new_obj);

                    Loader::add_object(&mut commands, &new_obj);
                } else {
                    log::warn!(
                        "{:?} fail to produce fleet, prefab id {:?} not found, ignoring production",
                        shipyard_id,
                        prefab_id
                    );
                }
            }
            ProductionResult::Producing => {}

            ProductionResult::NotProducing => {
                let (prefab_id, clean_on_build) = match shipyard.production_order {
                    ProductionOrder::None => {
                        log::trace!("{:?} no producing order, skipping", shipyard_id);
                        continue;
                    }
                    ProductionOrder::Next(prefab_id) => (prefab_id, true),
                    ProductionOrder::Random => {
                        let index = rand::thread_rng().gen_range(0..prefabs_candidates.len());
                        let (prefab_id, _) = prefabs_candidates[index];
                        (prefab_id, false)
                    }
                    ProductionOrder::RandomSelected(prefab_id) => (prefab_id, false),
                };

                let prefab = match prefabs_candidates.iter().find(|(e, _)| *e == prefab_id) {
                    Some((_, prefab)) => prefab,
                    None => {
                        log::warn!(
                            "shipyard could not find prefab from id {:?}, skipping",
                            prefab_id
                        );
                        continue;
                    }
                };

                let production_cost = match prefab.obj.production_cost.as_ref() {
                    Some(value) => value,
                    None => {
                        log::warn!(
                            "prefab_id {:?} do not have production cost, skipping",
                            prefab_id
                        );
                        continue;
                    }
                };

                // check if have enough resources
                if cargo.remove_all_or_none(&production_cost.cost).is_ok() {
                    // setup completion
                    shipyard.current_production = Some(ShipyardProduction {
                        pending_work: production_cost.work,
                        prefab_id,
                    });

                    // update next order
                    if clean_on_build {
                        shipyard.production_order = ProductionOrder::None;
                    } else {
                        shipyard.production_order = ProductionOrder::Random;
                    }

                    // remove requesting orders
                    trade_order.remove_by_id(TRADE_ORDER_ID_SHIPYARD);

                    log::debug!(
                        "{:?} staring production of prefab {:?}, expected to be complete at {:?}, next order is {:?}",
                        shipyard_id,
                        prefab_id,
                        production_cost.work / shipyard.production,
                        shipyard.production_order,
                    );
                } else {
                    log::trace!(
                        "{:?} can not start production of {:?}, not enough resources",
                        shipyard_id,
                        prefab_id
                    );

                    if shipyard.dirt_trade_order {
                        // update trade orders
                        shipyard.dirt_trade_order = false;
                        trade_order.remove_by_id(TRADE_ORDER_ID_SHIPYARD);
                        let requested_wares = production_cost.cost.get_wares_id();
                        log::trace!(
                            "{:?} updating trading orders to request {:?} ",
                            shipyard_id,
                            requested_wares
                        );
                        for ware_id in requested_wares {
                            trade_order.add_request(TRADE_ORDER_ID_SHIPYARD, ware_id);
                        }
                    }
                }
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::game::bevy_utils::WorldExt;
    use crate::game::commands::Command;
    use crate::game::dock::HasDocking;
    use crate::game::events::GEvents;
    use crate::game::fleets::Fleet;
    use crate::game::label::Label;
    use crate::game::loader::Loader;
    use crate::game::locations::LocationDocked;
    use crate::game::objects::ObjId;
    use crate::game::order::TradeOrders;
    use crate::game::utils::DeltaTime;
    use crate::game::wares::{Volume, WareAmount, WareId};
    use bevy_ecs::system::{RunSystemOnce, SystemState};

    use super::*;

    const TOTAL_WORK: f32 = 5.0;
    const PENDING_WORK_AFTER_SECOND: f32 = TOTAL_WORK - 1.0;
    const REQUIRE_CARGO: Volume = 50;
    const NOT_ENOUGH_CARGO: Volume = REQUIRE_CARGO - 5;
    const TIME_TO_WORK_COMPLETE: DeltaTime = DeltaTime(TOTAL_WORK);
    const NOT_ENOUGH_TIME: DeltaTime = DeltaTime(1.0);

    #[test]
    fn test_shipyard_system_should_not_start_production_without_enough_cargo() {
        let (mut world, (shipyard_id, ware_id, prefab_id)) =
            scenery(NOT_ENOUGH_TIME, NOT_ENOUGH_CARGO, None, |prefab_id| {
                ProductionOrder::Next(prefab_id)
            });
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, NOT_ENOUGH_CARGO);
        assert_shipyard_production(&mut world, shipyard_id, None);
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::Next(prefab_id));
        assert_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_but_production_not_selected(
    ) {
        let (mut world, (shipyard_id, ware_id, _prefab_id)) =
            scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |_| {
                ProductionOrder::None
            });
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&mut world, shipyard_id, None);
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::None);
        assert_no_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_should_start_production_with_selected_order_and_current_order_changed_to_none(
    ) {
        let (mut world, (shipyard_id, ware_id, _prefab_id)) =
            scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |prefab_id| {
                ProductionOrder::Next(prefab_id)
            });
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, 0);
        assert_shipyard_production(&mut world, shipyard_id, Some(TOTAL_WORK));
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::None);
        assert_no_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_with_random_order_should_start_production_and_keep_order_at_random() {
        let (mut world, (shipyard_id, ware_id, _prefab_id)) =
            scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, None, |_| {
                ProductionOrder::Random
            });
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, 0);
        assert_shipyard_production(&mut world, shipyard_id, Some(TOTAL_WORK));
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::Random);
        assert_no_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_in_parallel_with_enough_cargo_but_already_producing(
    ) {
        let (mut world, (shipyard_id, ware_id, _prefab_id)) =
            scenery(NOT_ENOUGH_TIME, REQUIRE_CARGO, Some(TOTAL_WORK), |_| {
                ProductionOrder::None
            });
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&mut world, shipyard_id, Some(PENDING_WORK_AFTER_SECOND));
        assert_no_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_should_keep_queued_next_order_during_production() {
        let (mut world, (shipyard_id, ware_id, prefab_id)) = scenery(
            NOT_ENOUGH_TIME,
            REQUIRE_CARGO,
            Some(TOTAL_WORK),
            |prefab_id| ProductionOrder::Next(prefab_id),
        );
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&mut world, shipyard_id, Some(PENDING_WORK_AFTER_SECOND));
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::Next(prefab_id));
        assert_no_buy_order(&mut world, shipyard_id);
    }

    #[test]
    fn test_shipyard_system_should_complete_production() {
        let (mut world, (shipyard_id, _ware_id, _prefab_id)) =
            scenery(TIME_TO_WORK_COMPLETE, 0, Some(TOTAL_WORK), |_| {
                ProductionOrder::None
            });
        assert_shipyard_production(&mut world, shipyard_id, None);
        assert_no_buy_order(&mut world, shipyard_id);
        assert_fleet_created(&mut world, shipyard_id);
    }

    fn assert_fleet_created(world: &mut World, shipyard_id: ObjId) {
        // search for added object
        let mut ss: SystemState<Query<&LocationDocked, (With<Fleet>, With<Command>)>> =
            SystemState::new(world);
        let mut query = ss.get(world);
        let docked = query.iter().next().expect("fleet not found");
        assert_eq!(shipyard_id, docked.parent_id);
    }

    #[test]
    fn test_shipyard_system_on_completion_should_not_start_next_selected_order_until_next_tick() {
        let (mut world, (shipyard_id, ware_id, prefab_id)) = scenery(
            TIME_TO_WORK_COMPLETE,
            REQUIRE_CARGO,
            Some(TOTAL_WORK),
            |prefab_id| ProductionOrder::Next(prefab_id),
        );
        assert_shipyard_production(&mut world, shipyard_id, None);
        assert_shipyard_selected(&mut world, shipyard_id, ProductionOrder::Next(prefab_id));
        assert_shipyard_cargo(&mut world, shipyard_id, ware_id, REQUIRE_CARGO);
        assert_no_buy_order(&mut world, shipyard_id);
    }

    fn assert_shipyard_cargo(world: &mut World, entity: Entity, ware_id: WareId, expected: Volume) {
        let current_cargo = world
            .get_entity(entity)
            .unwrap()
            .get::<Cargo>()
            .unwrap()
            .get_amount(ware_id);
        assert_eq!(expected, current_cargo);
    }

    fn assert_shipyard_production(world: &mut World, entity: Entity, expected: Option<WorkUnit>) {
        let current_production = world
            .get::<Shipyard>(entity)
            .unwrap()
            .current_production
            .as_ref()
            .map(|i| i.pending_work);
        assert_eq!(expected, current_production);
    }

    fn assert_shipyard_selected(
        world: &mut World,
        entity: Entity,
        expected_selected: ProductionOrder,
    ) {
        let current_production = world.get::<Shipyard>(entity).unwrap().production_order;
        assert_eq!(expected_selected, current_production);
    }

    fn assert_buy_order(world: &mut World, shipyard_id: Entity) {
        let orders = world
            .get::<TradeOrders>(shipyard_id)
            .expect("orders not found for shipyard")
            .clone();

        assert!(orders.is_requesting());
        assert!(!orders.is_provide());
    }

    fn assert_no_buy_order(world: &mut World, shipyard_id: Entity) {
        assert!(
            world.get::<TradeOrders>(shipyard_id).unwrap().is_empty(),
            "trade orders are not empty"
        );
    }

    /// returns the world and shipyard entity
    fn scenery(
        system_update_delta_time: DeltaTime,
        station_current_cargo_amount: Volume,
        current_production: Option<WorkUnit>,
        next_order: fn(PrefabId) -> ProductionOrder,
    ) -> (World, (Entity, WareId, PrefabId)) {
        let mut world = World::new();
        world.insert_resource(GEvents::default());

        let ware_id = world.spawn_empty().insert(Label::from("ore")).id();

        let prefab_id = world.run_commands(|mut commands| {
            // create station prefab to check we never building it by mistake
            Loader::add_prefab(
                &mut commands,
                "station",
                "station",
                Loader::new_station(),
                false,
                true,
            );

            // add a fleet prefab
            let new_obj = Loader::new_ship(2.0, "fleet".to_string())
                .with_command(Command::mine())
                .with_production_cost(TOTAL_WORK, vec![WareAmount::new(ware_id, REQUIRE_CARGO)]);

            Loader::add_prefab(&mut commands, "fleet", "fleet", new_obj, true, false)
        });

        assert!(station_current_cargo_amount < 1000);
        let mut cargo = Cargo::new(1000);
        if station_current_cargo_amount > 0 {
            cargo.add(ware_id, station_current_cargo_amount).unwrap();
        }

        world.insert_resource(system_update_delta_time);

        let mut shipyard = Shipyard::new();
        shipyard.production_order = next_order(prefab_id);
        shipyard.current_production = current_production.map(|pending_work| ShipyardProduction {
            pending_work,
            prefab_id,
        });
        shipyard.dirt_trade_order = true;

        let shipyard_id = world
            .spawn_empty()
            .insert(Label::from("shipyard"))
            .insert(cargo)
            .insert(shipyard.clone())
            .insert(HasDocking::default())
            .insert(TradeOrders::default())
            .id();
        log::trace!("creating shipyard {:?} {:?}", shipyard_id, shipyard);

        world.run_system_once(system_shipyard);

        (world, (shipyard_id, ware_id, prefab_id))
    }
}
