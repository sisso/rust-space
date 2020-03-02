use specs::prelude::*;
use crate::game::wares::Cargo;
use crate::game::new_obj::NewObj;
use crate::utils::{Speed, TotalTime, DeltaTime};
use crate::game::{GameInitContext, RequireInitializer};

const REQUIRE_CARGO: f32 = 5.0;
const PRODUCTION_TIME: f32 = 5.0;

#[derive(Debug,Clone,Component)]
pub struct Shipyard {
    production: Option<TotalTime>,
}

impl Shipyard {
    pub fn new() -> Self {
        Shipyard {
            production: None,
        }
    }
}

impl RequireInitializer for Shipyard {
    fn init(context: &mut GameInitContext) {
        context.dispatcher.add(ShipyardSystem, "shipyard_system", &[]);
    }
}

pub struct ShipyardSystem;

impl<'a> System<'a> for ShipyardSystem {
    type SystemData = (
        Read<'a, TotalTime>,
        Entities<'a>,
        WriteStorage<'a, Cargo>,
        WriteStorage<'a, Shipyard>,
        WriteStorage<'a, NewObj>
    );

    fn run(&mut self, data: Self::SystemData) {
        debug!("running");

        let (
            total_time,
            entities,
            mut cargos,
            mut shipyards,
            mut new_objects
        ) = data;

        let mut to_add = vec![];

        for (entity, cargo, shipyard) in (&*entities, &mut cargos, &mut shipyards).join() {
            match shipyard.production {
                Some(time) if total_time.is_after(time) => {
                    shipyard.production = None;

                    let new_obj = NewObj::new()
                        .with_ai()
                        .with_command_mine()
                        .with_cargo(10.0)
                        .with_speed(Speed(2.0))
                        .at_dock(entity);

                    to_add.push(new_obj);

                    debug!("{:?} complete production, scheduling new object", entity);
                },
                Some(_) => {
                    // still producing
                },
                None => {
                    if cargo.get_current() >= REQUIRE_CARGO {
                        let ware_id = cargo.get_wares().into_iter().next().unwrap();
                        cargo.remove(*ware_id, REQUIRE_CARGO).unwrap();

                        let ready_time = total_time.add(DeltaTime(PRODUCTION_TIME));
                        shipyard.production = Some(ready_time);

                        debug!("{:?} staring production, will be ready at {:?}", entity, ready_time);
                    }
                },
            }
        }

        // let new_objects = &mut new_objects;
        for obj in to_add {
            entities.build_entity()
                .with(obj, &mut new_objects)
                .build();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;
    use crate::game::wares::WareId;
    use std::borrow::Borrow;
    use crate::game::locations::Location;
    use crate::game::events::EventKind::Add;
    use crate::utils::V2;
    use crate::space_outputs_generated::space_data::EntityKind::Station;
    use crate::game::commands::CommandMine;

    #[test]
    fn test_shipyard_system_should_not_start_production_without_enough_cargo() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO - 0.5, None);
        assert_shipyard_cargo(&world, entity, ware_id, REQUIRE_CARGO - 0.5);
        assert_shipyard_production(&world, entity, None);
    }

    #[test]
    fn test_shipyard_system_should_start_production_with_enough_cargo() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, None);
        assert_shipyard_cargo(&world, entity,  ware_id,0.0);
        assert_shipyard_production(&world, entity, Some(TotalTime(PRODUCTION_TIME as f64)));
    }

    #[test]
    fn test_shipyard_system_should_not_start_production_with_enough_cargo_and_already_producing() {
        let (world, (entity, ware_id)) = scenery(0.0, REQUIRE_CARGO, Some(1.0));
        assert_shipyard_cargo(&world, entity,  ware_id,REQUIRE_CARGO);
        assert_shipyard_production(&world, entity, Some(TotalTime(1.0)));
    }

    #[test]
    fn test_shipyard_system_should_complete_production() {
        let (world, (entity, ware_id)) = scenery(2.0, 0.0, Some(1.0));
        assert_shipyard_production(&world, entity, None);

        let storage = &world.read_storage::<NewObj>();

        let new_obj: &NewObj = storage.as_slice()
            .iter()
            .next()
            .unwrap();

        assert!(new_obj.ai);
        assert!(new_obj.speed.is_some());

        match &new_obj.location {
            Some(Location::Dock { docked_id }) => {
                assert_eq!(*docked_id, entity);
            },
            other => {
                panic!("unexpected location {:?}", other);
            }
        }

        assert!(new_obj.command_mine);
    }

    fn assert_shipyard_cargo(world: &World, entity: Entity, ware_id: WareId, expected: f32) {
        let current_cargo = world.read_storage::<Cargo>()
            .get(entity)
            .unwrap()
            .get_amount(ware_id);

        assert_eq!(current_cargo, expected);
    }

    fn assert_shipyard_production(world: &World, entity: Entity, expected: Option<TotalTime>) {
        let current_production = world.read_storage::<Shipyard>().get(entity)
            .unwrap()
            .production
            .clone()
            .map(|i| i.as_u64());

        assert_eq!(current_production, expected.map(|i| i.as_u64()));
    }

    /// returns the world and shipyard entity
    fn scenery(total_time: f64, cargo_amount: f32, production: Option<f64>) -> (World, (Entity, WareId)) {
        test_system(ShipyardSystem, move |world| {
            let ware_id = world.create_entity().build();

            let mut cargo = Cargo::new(100.0);
            if cargo_amount > 0.0 {
                cargo.add(ware_id, cargo_amount).unwrap();
            }

            world.insert(TotalTime(total_time));

            let entity = world
                .create_entity()
                .with(cargo)
                .with(Shipyard {
                    production: production.map(|v| TotalTime(v))
                })
                .build();

            (entity, ware_id)
        })
    }
}