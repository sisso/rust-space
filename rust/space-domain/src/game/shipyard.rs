use rand::Rng;
use specs::prelude::*;

use crate::game::blueprint::Blueprint;
use crate::game::new_obj::NewObj;
use crate::game::prefab::Prefab;
use crate::game::wares::{Cargo, WareAmount};
use crate::game::{prefab, GameInitContext, RequireInitializer};
use crate::utils::{DeltaTime, Speed, TotalTime};

#[derive(Debug, Clone)]
struct ShipyardProduction {
    complete_at: TotalTime,
    blueprint_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProductionOrder {
    None,
    Next(usize),
    Random,
}

#[derive(Debug, Clone, Component)]
pub struct Shipyard {
    pub blueprints: Vec<Blueprint>,
    pub order: ProductionOrder,
    current_production: Option<ShipyardProduction>,
}

impl Shipyard {
    pub fn new(blueprints: Vec<Blueprint>) -> Self {
        Self {
            blueprints,
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

impl<'a> System<'a> for ShipyardSystem {
    type SystemData = (
        Read<'a, TotalTime>,
        Entities<'a>,
        WriteStorage<'a, Cargo>,
        WriteStorage<'a, Shipyard>,
        WriteStorage<'a, NewObj>,
        ReadStorage<'a, Prefab>,
    );

    fn run(
        &mut self,
        (total_time, entities, mut cargos, mut shipyards, mut new_objects, prefabs): Self::SystemData,
    ) {
        log::trace!("running");

        let mut produced_fleets = vec![];

        for (entity, cargo, shipyard) in (&*entities, &mut cargos, &mut shipyards).join() {
            if shipyard.blueprints.len() == 0 {
                continue;
            }

            match &shipyard.current_production {
                Some(sp) if total_time.is_after(sp.complete_at) => {
                    let index = sp.blueprint_index;
                    shipyard.current_production = None;

                    let produced_prefab_id = shipyard.blueprints[index].output;
                    if let Some(mut new_obj) = prefab::get_by_id(&prefabs, produced_prefab_id) {
                        // put into shipyard
                        new_obj = new_obj.at_dock(entity);
                        produced_fleets.push(new_obj);
                        log::debug!("{:?} complete production, scheduling new object", entity);
                    } else {
                        log::warn!(
                            "{:?} fail to produce fleet, prefab id {:?} not found",
                            entity,
                            produced_prefab_id
                        );
                    }
                }
                Some(_) => {
                    // still producing
                }
                None => {
                    let (index, clean_on_build) = match shipyard.order {
                        ProductionOrder::None => continue,
                        ProductionOrder::Next(index) => (index, true),
                        ProductionOrder::Random => {
                            let index = rand::thread_rng().gen_range(0..shipyard.blueprints.len());
                            (index, false)
                        }
                    };

                    // chose one random blueprint to produce and check if have enough resources
                    if cargo.remove_all(&shipyard.blueprints[index].input).is_ok() {
                        if clean_on_build {
                            shipyard.order = ProductionOrder::None;
                        }

                        // setup completion
                        let complete_time = total_time.add(shipyard.blueprints[index].time);
                        shipyard.current_production = Some(ShipyardProduction {
                            complete_at: complete_time,
                            blueprint_index: index,
                        });

                        log::debug!(
                            "{:?} staring production of bluprint {:?} will be complete at {:?}",
                            entity,
                            shipyard.blueprints[index].label,
                            complete_time,
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
    use crate::game::code::CodeRef;
    use crate::game::loader::Loader;
    use crate::game::locations::Location;
    use crate::game::wares::{Volume, Ware, WareId};
    use crate::test::test_system;

    use super::*;

    const PRODUCTION_TIME: f32 = 5.0;
    const REQUIRE_CARGO: Volume = 50;

    #[test]
    fn test_shipyard_system_should_not_start_production_without_enough_cargo() {
        let (world, (entity, ware_id)) =
            scenery(0.0, REQUIRE_CARGO - 5, None, ProductionOrder::Next(0));
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO - 5);
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(0));
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_but_production_not_selected(
    ) {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, None, ProductionOrder::None);
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::None);
    }

    #[test]
    fn test_shipyard_system_should_start_production_with_selected_blueprint_should_cleanup_order() {
        let (world, (entity, ware_id)) =
            scenery(0.0, REQUIRE_CARGO, None, ProductionOrder::Next(0));
        assert_shipyard_cargo(&world, entity, ware_id, 0);
        assert_shipyard_production(&world, entity, Some(TotalTime(PRODUCTION_TIME as f64)));
        assert_shipyard_selected(&world, entity, ProductionOrder::None);
    }

    #[test]
    fn test_shipyard_system_with_random_order_should_start_production_and_keep_order() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, None, ProductionOrder::Random);
        assert_shipyard_cargo(&world, entity, ware_id, 0);
        assert_shipyard_production(&world, entity, Some(TotalTime(PRODUCTION_TIME as f64)));
        assert_shipyard_selected(&world, entity, ProductionOrder::Random);
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_and_already_producing() {
        let (world, (entity, ware_id)) =
            scenery(0.0, REQUIRE_CARGO, Some(1.0), ProductionOrder::None);
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(TotalTime(1.0)));
    }

    #[test]
    fn test_shipyard_system_should_keep_select_bp_as_next_during_production() {
        let (world, (entity, ware_id)) =
            scenery(0.0, REQUIRE_CARGO, Some(1.0), ProductionOrder::Next(0));
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(TotalTime(1.0)));
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(0));
    }

    #[test]
    fn test_shipyard_system_should_complete_production() {
        let (world, (entity, _ware_id)) = scenery(2.0, 0, Some(1.0), ProductionOrder::None);
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
        let (world, (entity, ware_id)) =
            scenery(2.0, REQUIRE_CARGO, Some(1.0), ProductionOrder::Next(0));
        assert_shipyard_production(&world, entity, None);
        assert_shipyard_selected(&world, entity, ProductionOrder::Next(0));
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
    }

    fn assert_shipyard_cargo(world: &World, entity: Entity, ware_id: WareId, expected: Volume) {
        let current_cargo = world
            .read_storage::<Cargo>()
            .get(entity)
            .unwrap()
            .get_amount(ware_id);

        assert_eq!(current_cargo, expected);
    }

    fn assert_shipyard_production(world: &World, entity: Entity, expected: Option<TotalTime>) {
        let current_production = world
            .read_storage::<Shipyard>()
            .get(entity)
            .unwrap()
            .current_production
            .as_ref()
            .map(|i| i.complete_at.as_u64());

        assert_eq!(current_production, expected.map(|i| i.as_u64()));
    }

    fn assert_shipyard_selected(world: &World, entity: Entity, expected_selected: ProductionOrder) {
        let current_production = world.read_storage::<Shipyard>().get(entity).unwrap().order;
        assert_eq!(expected_selected, current_production);
    }

    fn load_fleets_prefab(
        world: &mut World,
        ware_id: WareId,
        ware_amount: u32,
        time: f32,
        code: &CodeRef,
        command: Command,
    ) -> Blueprint {
        let new_obj = Loader::new_ship(2.0, code.to_string())
            .with_command(command)
            .with_ai();
        let prefab_id = Loader::add_prefab(world, code, new_obj);

        Blueprint {
            label: format!("Produce {code}"),
            input: vec![WareAmount::new(ware_id, ware_amount)],
            output: prefab_id,
            time: time.into(),
        }
    }

    /// returns the world and shipyard entity
    fn scenery(
        total_time: f64,
        cargo_amount: Volume,
        complete_time: Option<f64>,
        order: ProductionOrder,
    ) -> (World, (Entity, WareId)) {
        test_system(ShipyardSystem, move |world| {
            world.register::<HasCode>();

            let ware_id = world.create_entity().build();

            let blueprint = load_fleets_prefab(
                world,
                ware_id,
                REQUIRE_CARGO,
                PRODUCTION_TIME,
                "ware",
                Command::mine(),
            );

            let mut cargo = Cargo::new(1000);
            if cargo_amount > 0 {
                cargo.add(ware_id, cargo_amount).unwrap();
            }

            world.insert(TotalTime(total_time));

            let mut shipyard = Shipyard::new(vec![blueprint]);
            shipyard.order = order;
            shipyard.current_production = complete_time.map(|complete_time| ShipyardProduction {
                complete_at: TotalTime(complete_time),
                blueprint_index: 0,
            });

            let entity = world.create_entity().with(cargo).with(shipyard).build();

            (entity, ware_id)
        })
    }
}
