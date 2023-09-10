use rand::Rng;
use specs::prelude::*;

use crate::game::new_obj::NewObj;
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::wares::Cargo;
use crate::game::work::WorkUnit;
use crate::game::{prefab, GameInitContext, RequireInitializer};
use crate::utils::DeltaTime;

#[derive(Debug, Clone)]
struct ShipyardProduction {
    pending_work: WorkUnit,
    prefab_id: PrefabId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProductionOrder {
    None,
    Next(PrefabId),
    Random,
}

/// shipyard are attached to stations and can building ships
#[derive(Debug, Clone, Component)]
pub struct Shipyard {
    pub production: WorkUnit,
    pub order: ProductionOrder,
    current_production: Option<ShipyardProduction>,
}

impl Shipyard {
    pub fn new() -> Self {
        Self {
            production: 1.0,
            order: ProductionOrder::Random,
            current_production: None,
        }
    }

    pub fn is_producing(&self) -> bool {
        self.current_production.is_some()
    }
}

impl RequireInitializer for Shipyard {
    fn init(context: &mut GameInitContext) {
        context
            .dispatcher
            .add(ShipyardSystem, "shipyard_system", &[]);
    }
}

pub struct ShipyardSystem;

/// automatically produce one of available fleets at random, once all resources are in place, create
/// a new fleet and start the process
impl<'a> System<'a> for ShipyardSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Entities<'a>,
        WriteStorage<'a, Cargo>,
        WriteStorage<'a, Shipyard>,
        WriteStorage<'a, NewObj>,
        ReadStorage<'a, Prefab>,
    );

    fn run(
        &mut self,
        (delta_time, entities, mut cargos, mut shipyards, mut new_objects, prefabs): Self::SystemData,
    ) {
        log::trace!("running");

        // collect all prefabs as candidates for random production
        let prefabs_candidates: Vec<_> = (&entities, &prefabs).join().collect();

        let mut produced_fleets = vec![];

        for (entity, cargo, shipyard) in (&*entities, &mut cargos, &mut shipyards).join() {
            match &mut shipyard.current_production {
                Some(sp) if sp.pending_work - shipyard.production * delta_time.as_f32() <= 0.0 => {
                    // move out the reference to allow us to change current production
                    let prefab_id = sp.prefab_id;

                    // complete current production
                    shipyard.current_production = None;

                    // create produced prefab
                    if let Some(mut new_obj) = prefab::get_by_id(&prefabs, prefab_id) {
                        // put into shipyard
                        new_obj = new_obj.at_dock(entity);
                        produced_fleets.push(new_obj);
                        log::debug!("{:?} complete production, scheduling new object", entity);
                    } else {
                        log::warn!(
                            "{:?} fail to produce fleet, prefab id {:?} not found",
                            entity,
                            prefab_id
                        );
                    }
                }
                Some(sp) => {
                    // deducted work done
                    sp.pending_work -= shipyard.production;
                }
                None => {
                    let (prefab_id, prefab, clean_on_build) = match shipyard.order {
                        ProductionOrder::None => continue,
                        ProductionOrder::Next(prefab_id) => {
                            let (_, prefab) =
                                match prefabs_candidates.iter().find(|(e, _)| *e == prefab_id) {
                                    Some(v) => v,
                                    None => {
                                        log::warn!(
                                            "shipyard could not find prefab from id {:?}, skipping",
                                            prefab_id
                                        );
                                        continue;
                                    }
                                };

                            (prefab_id, *prefab, true)
                        }
                        ProductionOrder::Random => {
                            let index = rand::thread_rng().gen_range(0..prefabs_candidates.len());
                            let (prefab_id, prefab) = prefabs_candidates[index];
                            (prefab_id, prefab, false)
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

                    // chose one random blueprint to produce and check if have enough resources
                    if cargo.remove_all(&production_cost.cost).is_ok() {
                        if clean_on_build {
                            shipyard.order = ProductionOrder::None;
                        }

                        // setup completion
                        shipyard.current_production = Some(ShipyardProduction {
                            pending_work: production_cost.work,
                            prefab_id,
                        });

                        log::debug!(
                            "{:?} staring production of prefab {:?}, expected to be complete at {:?}",
                            entity,
                            prefab_id,
                            production_cost.work / shipyard.production,
                        );
                    }
                }
            }
        }

        // let new_objects = &mut new_objects;
        for obj in produced_fleets {
            entities.build_entity().with(obj, &mut new_objects).build();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::code::HasCode;
    use crate::game::commands::Command;
    use crate::game::loader::Loader;
    use crate::game::locations::Location;
    use crate::game::wares::{Volume, WareAmount, WareId};
    use crate::test::test_system;
    use crate::utils::DeltaTime;

    use super::*;

    const TOTAL_WORK: f32 = 5.0;
    const PENDING_WORK_AFTER_SECOND: f32 = TOTAL_WORK - 1.0;
    const REQUIRE_CARGO: Volume = 50;
    const NOT_ENOUGH_CARGO: Volume = REQUIRE_CARGO - 5;
    const TIME_TO_WORK_COMPLETE: DeltaTime = DeltaTime(TOTAL_WORK);

    #[test]
    fn test_shipyard_system_should_not_start_production_without_enough_cargo() {
        let (world, (entity, ware_id, prefab_id)) =
            scenery(DeltaTime(1.0), NOT_ENOUGH_CARGO, None, |prefab_id| {
                ProductionOrder::Next(prefab_id)
            });
        assert_shipyard_cargo(&world, entity, ware_id, NOT_ENOUGH_CARGO);
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(prefab_id));
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_but_production_not_selected(
    ) {
        let (world, (entity, ware_id, _prefab_id)) =
            scenery(DeltaTime(1.0), REQUIRE_CARGO, None, |_| {
                ProductionOrder::None
            });
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::None);
    }

    #[test]
    fn test_shipyard_system_should_start_production_with_selected_order_and_current_order_changed_to_none(
    ) {
        let (world, (entity, ware_id, _prefab_id)) =
            scenery(DeltaTime(1.0), REQUIRE_CARGO, None, |prefab_id| {
                ProductionOrder::Next(prefab_id)
            });
        assert_shipyard_cargo(&world, entity, ware_id, 0);
        assert_shipyard_production(&world, entity, Some(TOTAL_WORK));
        assert_shipyard_selected(&world, entity, ProductionOrder::None);
    }

    #[test]
    fn test_shipyard_system_with_random_order_should_start_production_and_keep_order_at_random() {
        let (world, (entity, ware_id, _prefab_id)) =
            scenery(DeltaTime(1.0), REQUIRE_CARGO, None, |_| {
                ProductionOrder::Random
            });
        assert_shipyard_cargo(&world, entity, ware_id, 0);
        assert_shipyard_production(&world, entity, Some(TOTAL_WORK));
        assert_shipyard_selected(&world, entity, ProductionOrder::Random);
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_in_parallel_with_enough_cargo_but_already_producing(
    ) {
        let (world, (entity, ware_id, _prefab_id)) =
            scenery(DeltaTime(1.0), REQUIRE_CARGO, Some(TOTAL_WORK), |_| {
                ProductionOrder::None
            });
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(PENDING_WORK_AFTER_SECOND));
    }

    #[test]
    fn test_shipyard_system_should_keep_queued_next_order_during_production() {
        let (world, (entity, ware_id, prefab_id)) = scenery(
            DeltaTime(1.0),
            REQUIRE_CARGO,
            Some(TOTAL_WORK),
            |prefab_id| ProductionOrder::Next(prefab_id),
        );
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(PENDING_WORK_AFTER_SECOND));
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(prefab_id));
    }

    #[test]
    fn test_shipyard_system_should_complete_production() {
        let (world, (entity, _ware_id, _prefab_id)) =
            scenery(TIME_TO_WORK_COMPLETE, 0, Some(TOTAL_WORK), |_| {
                ProductionOrder::None
            });
        assert_shipyard_production(&world, entity, None);

        let storage = &world.read_storage::<NewObj>();

        let new_obj: &NewObj = storage.as_slice().iter().last().unwrap();

        assert!(new_obj.ai);
        assert!(new_obj.speed.is_some());

        match &new_obj.location {
            Some(Location::Dock { docked_id }) => {
                assert_eq!(*docked_id, entity);
            }
            other => {
                panic!("unexpected location {:?}", other);
            }
        }

        // check command
        assert!(new_obj.command.is_some());
    }

    #[test]
    fn test_shipyard_system_on_completion_should_not_start_next_selected_bp_until_next_tick() {
        let (world, (entity, ware_id, prefab_id)) = scenery(
            TIME_TO_WORK_COMPLETE,
            REQUIRE_CARGO,
            Some(TOTAL_WORK),
            |prefab_id| ProductionOrder::Next(prefab_id),
        );
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(prefab_id));
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
    }

    fn assert_shipyard_cargo(world: &World, entity: Entity, ware_id: WareId, expected: Volume) {
        let current_cargo = world
            .read_storage::<Cargo>()
            .get(entity)
            .unwrap()
            .get_amount(ware_id);

        assert_eq!(expected, current_cargo);
    }

    fn assert_shipyard_production(world: &World, entity: Entity, expected: Option<WorkUnit>) {
        let current_production = world
            .read_storage::<Shipyard>()
            .get(entity)
            .unwrap()
            .current_production
            .as_ref()
            .map(|i| i.pending_work);

        assert_eq!(expected, current_production);
    }

    fn assert_shipyard_selected(world: &World, entity: Entity, expected_selected: ProductionOrder) {
        let current_production = world.read_storage::<Shipyard>().get(entity).unwrap().order;
        assert_eq!(expected_selected, current_production);
    }

    /// returns the world and shipyard entity
    fn scenery(
        system_update_delta_time: DeltaTime,
        station_current_cargo_amount: Volume,
        current_production: Option<WorkUnit>,
        next_order: fn(PrefabId) -> ProductionOrder,
    ) -> (World, (Entity, WareId, PrefabId)) {
        test_system(ShipyardSystem, move |world| {
            world.register::<HasCode>();

            let ware_id = world.create_entity().build();
            let new_obj = Loader::new_ship(2.0, "fleet".to_string())
                .with_command(Command::mine())
                .with_production_cost(TOTAL_WORK, vec![WareAmount::new(ware_id, REQUIRE_CARGO)])
                .with_ai();
            let prefab_id = Loader::add_prefab(world, "fleet", new_obj);

            // let blueprint = load_fleets_prefab(
            //     world,
            //     ware_id,
            //     REQUIRE_CARGO,
            //     PRODUCTION_TIME,
            //     "ware",
            //     Command::mine(),
            // );

            assert!(station_current_cargo_amount < 1000);
            let mut cargo = Cargo::new(1000);
            if station_current_cargo_amount > 0 {
                cargo.add(ware_id, station_current_cargo_amount).unwrap();
            }

            world.insert(system_update_delta_time);

            let mut shipyard = Shipyard::new();
            shipyard.order = next_order(prefab_id);
            shipyard.current_production =
                current_production.map(|pending_work| ShipyardProduction {
                    pending_work: pending_work,
                    prefab_id: prefab_id,
                });

            let entity = world.create_entity().with(cargo).with(shipyard).build();

            (entity, ware_id, prefab_id)
        })
    }
}
