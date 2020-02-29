use specs::prelude::*;
use crate::utils::{DeltaTime, TotalTime};
use std::collections::HashMap;
use crate::game::wares::{WareId, Cargo, WareAmount};
use crate::game::{RequireInitializer, GameInitContext};

#[derive(Debug, Clone)]
pub struct Production {
    pub input: Vec<WareAmount>,
    pub output: Vec<WareAmount>,
    pub time: DeltaTime,
}

#[derive(Debug,Clone,Component)]
pub struct Factory {
    pub production: Production,
    pub production_time: Option<TotalTime>,
}

impl Factory {
    pub fn new(production: Production) -> Self {
        Factory {
            production,
            production_time: None,
        }
    }
}

impl RequireInitializer for Factory {
    fn init(context: &mut GameInitContext) {
        context.dispatcher.add(FactorySystem, "factory_system", &[]);
    }
}

pub struct FactorySystem;

impl<'a> System<'a> for FactorySystem {
    type SystemData = (
        Read<'a, TotalTime>,
        Entities<'a>,
        WriteStorage<'a, Cargo>,
        WriteStorage<'a, Factory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        debug!("running");

        let (
            total_time,
            entities,
            mut cargos,
            mut factories,
        ) = data;

        let total_time = *total_time;

        for (entity, cargo, factory) in (&*entities, &mut cargos, &mut factories).join() {
           match factory.production_time {
               Some(time) if total_time.is_after(time) => {
                   // production ready
                   let total_produce = factory.production.output.iter()
                       .map(|WareAmount(_, amount)| amount)
                       .sum();

                   if cargo.free_space() > total_produce {
                       for WareAmount(ware_id, amount) in &factory.production.output {
                          cargo.add(*ware_id, *amount).unwrap();
                       }

                       factory.production_time = None;
                   }
               },

               Some(time) => {
                   // producing
               },

               None => {
                   let mut has_all_inputs = true;

                   // check if can produce
                   for WareAmount(ware_id, amount) in &factory.production.input {
                       if cargo.get_amount(*ware_id) < *amount {
                           has_all_inputs = false;
                       }
                   }

                   if has_all_inputs {
                       for WareAmount(ware_id, amount) in &factory.production.input {
                           cargo.remove(*ware_id, *amount).unwrap();
                       }

                       factory.production_time = Some(total_time.add(factory.production.time));
                   } else {
                       // not enough cargo
                   }
               }
           }
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

    const ORE_ID: WareId = WareId(0);
    const ENERGY_ID: WareId = WareId(1);
    const PLATE_ID: WareId = WareId(2);
    
    fn get_production() -> Production {
        Production {
            input: vec![WareAmount(ORE_ID, 1.0), WareAmount(ENERGY_ID, 10.0)],
            output: vec![WareAmount(PLATE_ID, 1.0)],
            time: DeltaTime(5.0),
        }
    }

    #[test]
    fn test_factory_system_should_not_start_production_without_enough_cargo() {
        run_factory(0.0, 0.0, 3.0, None, None, 0.0, 0.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_not_start_production_without_both_enough_cargo() {
        run_factory(2.0, 0.0, 3.0, None, None, 2.0, 0.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_start_production_with_both_cargo() {
        run_factory(2.0, 10.0, 3.0, None, Some(8.0), 0.0, 1.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_keep_producing() {
        run_factory(2.0, 10.0, 3.0, Some(8.0), Some(8.0), 2.0, 10.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_produce() {
       run_factory(0.0, 0.0, 9.0, Some(8.0), None, 0.0, 0.0, 1.0);
    }

    #[test]
    fn test_factory_system_should_not_complete_production_if_cargo_is_full() {
        unimplemented!()
    }

    fn run_factory(ore: f32,
                   energy: f32,
                   total_time: f64,
                   production_time: Option<f64>,
                   expect_produce_at: Option<f64>,
                   expected_ore: f32,
                   expected_energy: f32,
                   expected_plates: f32) {
       let (world, entity) = test_system(FactorySystem, move |world| {
           let mut cargo = Cargo::new(100.0);
           cargo.add(ORE_ID, ore).unwrap();
           cargo.add(ENERGY_ID, energy).unwrap();

           world.insert(TotalTime(total_time));

           let entity = world
               .create_entity()
               .with(cargo)
               .with(Factory {
                   production: get_production(),
                   production_time: production_time.map(|time| TotalTime(time)),
               })
               .build();

           entity
       });

        let cargo = world.read_storage::<Cargo>().get(entity).unwrap().clone();
        assert_eq!(cargo.get_amount(PLATE_ID), expected_plates);

        let factory = world.read_storage::<Factory>().get(entity).unwrap().clone();
        assert_eq!(factory.production_time.map(|i| i.as_f64()), expect_produce_at);
    }
}
