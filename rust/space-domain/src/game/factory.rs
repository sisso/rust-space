use specs::prelude::*;
use crate::game::wares::Cargo;
use crate::game::new_obj::NewObj;
use crate::utils::Speed;

const REQUIRE_CARGO: f32 = 5.0;

#[derive(Debug,Clone,Component)]
pub struct Factory {

}

impl Factory {
    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        dispatcher.add(FactorySystem, "factory_system", &[]);
    }
}

pub struct FactorySystem;

impl<'a> System<'a> for FactorySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Cargo>,
        ReadStorage<'a, Factory>,
        WriteStorage<'a, NewObj>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut cargos, factories, mut new_objects) = data;

        let mut to_add = vec![];

        for (entity, cargo, _) in (&*entities, &mut cargos, &factories).join() {
            // TODO: only check for correct resources
            if cargo.get_current() > REQUIRE_CARGO {
                let ware_id = cargo.get_wares().into_iter().next().unwrap();
                cargo.remove(*ware_id, REQUIRE_CARGO).unwrap();

                let new_obj = NewObj::new()
                    .with_ai()
                    .with_command_mine()
                    .with_cargo(10.0)
                    .with_speed(Speed(2.0))
                    .at_dock(entity);

                to_add.push(new_obj);
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

    const WARE_ID: WareId = WareId(0);

    #[test]
    fn test_factory_system_should_create_new_miners_with_amount_of_ore() {
        test_one(REQUIRE_CARGO * 2.0, true, REQUIRE_CARGO);
    }

    #[test]
    fn test_factory_system_should_not_spawn_miner_without_enough_cargo() {
        test_one(REQUIRE_CARGO - 0.5 , false, REQUIRE_CARGO - 0.5);
    }

    /// returns the world and factory entity
    fn test_one(cargo_amount: f32, expected_new_miner: bool, expected_cargo: f32) {
        let (world, entity) = test_system(FactorySystem, move |world| {
            let mut cargo = Cargo::new(100.0);
            if cargo_amount > 0.0 {
                cargo.add(WARE_ID, cargo_amount);
            }

            let entity = world
                .create_entity()
                .with(cargo)
                .with(Factory {})
                .build();

            entity
        });

        let storage = &world.read_storage::<NewObj>();
        if expected_new_miner {
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
        } else {
            assert_eq!(0, storage.count());
        }

        let current_cargo = world.read_storage::<Cargo>()
            .get(entity)
            .unwrap()
            .get_amount(WARE_ID);

        assert_eq!(expected_cargo, current_cargo);
    }
}