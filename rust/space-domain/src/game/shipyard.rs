use specs::prelude::*;
use crate::game::wares::{Cargo, WareId, WareAmount};
use crate::game::new_obj::NewObj;
use crate::utils::{Speed, TotalTime, DeltaTime};
use crate::game::{GameInitContext, RequireInitializer};

#[derive(Debug,Clone,Component)]
pub struct Shipyard {
    pub input: WareAmount,
    pub production_time: DeltaTime,
    current_production: Option<TotalTime>,
}

impl Shipyard {
    pub fn new(input: WareAmount, production_time: DeltaTime) -> Self {
        Shipyard {
            input,
            production_time,
            current_production: None,
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
            match shipyard.current_production {
                Some(time) if total_time.is_after(time) => {
                    shipyard.current_production = None;

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
                    let ware_id = shipyard.input.get_ware_id();
                    let amount = shipyard.input.get_amount();

                    if cargo.get_amount(ware_id) >= amount {
                        cargo.remove(ware_id, amount).unwrap();

                        let ready_time = total_time.add(shipyard.production_time);
                        shipyard.current_production = Some(ready_time);

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

    const PRODUCTION_TIME: f32 = 5.0;
    const REQUIRE_CARGO: f32 = 5.0;


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
            .current_production
            .clone()
            .map(|i| i.as_u64());

        assert_eq!(current_production, expected.map(|i| i.as_u64()));
    }

    /// returns the world and shipyard entity
    fn scenery(total_time: f64, cargo_amount: f32, current_production: Option<f64>) -> (World, (Entity, WareId)) {
        test_system(ShipyardSystem, move |world| {
            let ware_id = world.create_entity().build();

            let mut cargo = Cargo::new(100.0);
            if cargo_amount > 0.0 {
                cargo.add(ware_id, cargo_amount).unwrap();
            }

            world.insert(TotalTime(total_time));

            let mut shipyard = Shipyard::new(WareAmount(ware_id, REQUIRE_CARGO), DeltaTime(PRODUCTION_TIME));
            shipyard.current_production = current_production.map(|i| TotalTime(i));

            let entity = world
                .create_entity()
                .with(cargo)
                .with(shipyard)
                .build();

            (entity, ware_id)
        })
    }
}