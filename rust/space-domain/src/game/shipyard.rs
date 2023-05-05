use rand::{Rng, RngCore};
use specs::prelude::*;
use std::fs::read;

use crate::game::code::{Code, HasCode};
use crate::game::commands::Command;
use crate::game::new_obj::NewObj;
use crate::game::prefab::Prefab;
use crate::game::wares::{Cargo, WareAmount};
use crate::game::{prefab, GameInitContext, RequireInitializer};
use crate::utils::{DeltaTime, Speed, TotalTime};

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub label: String,
    pub input: Vec<WareAmount>,
    pub output: Code,
    pub time: DeltaTime,
}

#[derive(Debug, Clone)]
struct ShipyardProduction {
    complete_at: TotalTime,
    blueprint_index: usize,
}

#[derive(Debug, Clone, Component)]
pub struct Shipyard {
    pub blueprints: Vec<Blueprint>,
    current_production: Option<ShipyardProduction>,
}

impl Shipyard {
    pub fn new(blueprints: Vec<Blueprint>) -> Self {
        Self {
            blueprints,
            current_production: None,
        }
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
        ReadStorage<'a, HasCode>,
        ReadStorage<'a, Prefab>,
    );

    fn run(
        &mut self,
        (total_time, entities, mut cargos, mut shipyards, mut new_objects, codes, prefabs): Self::SystemData,
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

                    let produced_code = shipyard.blueprints[index].output.as_str();
                    if let Some(new_obj) =
                        prefab::find_new_obj_by_code(&entities, &codes, &prefabs, produced_code)
                    {
                        produced_fleets.push(new_obj);
                        log::debug!("{:?} complete production, scheduling new object", entity);
                    } else {
                        log::warn!(
                            "{:?} fail to produce fleet, prefab {:?} not found",
                            entity,
                            produced_code
                        );
                    }
                }
                Some(_) => {
                    // still producing
                }
                None => {
                    // chose one random blueprint to produce and check if have enough resources
                    let index = rand::thread_rng().gen_range(0..shipyard.blueprints.len());
                    let ware_id = shipyard.blueprints[index].input.get_ware_id();
                    let amount = shipyard.blueprints[index].input.get_amount();

                    if cargo.get_amount(ware_id) >= amount {
                        cargo.remove(ware_id, amount).unwrap();

                        let complete_time = total_time.add(shipyard.production_time);
                        shipyard.current_production = Some(ShipyardProduction {
                            complete_at: complete_time,
                            blueprint_index: index,
                        });

                        log::debug!(
                            "{:?} staring production of bluprint {:?} will be complete at {:?}",
                            entity,
                            shipyard.bluprints[index].label,
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
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO - 5, None);
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO - 5);
        assert_shipyard_production(&world, entity, None);
    }

    #[test]
    fn test_shipyard_system_should_start_production_with_enough_cargo() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, None);
        assert_shipyard_cargo(&world, entity, ware_id, 0);
        assert_shipyard_production(&world, entity, Some(TotalTime(PRODUCTION_TIME as f64)));
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_and_already_producing() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, Some(1.0));
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(TotalTime(1.0)));
    }

    #[test]
    fn test_shipyard_system_should_complete_production() {
        let (world, (entity, _ware_id)) = scenery(2.0, 0, Some(1.0));
        assert_shipyard_production(&world, entity, None);

        let storage = &world.read_storage::<NewObj>();

        let new_obj: &NewObj = storage.as_slice().iter().next().unwrap();

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
            .clone()
            .map(|i| i.as_u64());

        assert_eq!(current_production, expected.map(|i| i.as_u64()));
    }

    fn load_fleets_prefab(
        world: &mut World,
        ware_id: WareId,
        ware_amount: u32,
        time: f32,
        code: &CodeRef,
        command: Command,
    ) -> Blueprint {
        let new_obj = Loader::new_ship(2.0, code.to_string()).with_command(command);
        Loader::add_prefab(world, code, new_obj);

        Blueprint {
            label: format!("Produce {code}"),
            input: vec![WareAmount::new(ware_id, ware_amount)],
            output: code.to_string(),
            time: time.into(),
        }
    }

    /// returns the world and shipyard entity
    fn scenery(
        total_time: f64,
        cargo_amount: Volume,
        current_production: Option<f64>,
    ) -> (World, (Entity, WareId)) {
        test_system(ShipyardSystem, move |world| {
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
            shipyard.current_production = current_production.map(|i| TotalTime(i));

            let entity = world.create_entity().with(cargo).with(shipyard).build();

            (entity, ware_id)
        })
    }
}
