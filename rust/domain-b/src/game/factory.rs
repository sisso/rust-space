use crate::game::utils::{DeltaTime, TotalTime};
use crate::game::wares::{Cargo, WareAmount, WareId};
use bevy_ecs::prelude::*;

#[derive(Debug, Clone)]
pub struct Receipt {
    pub label: String,
    pub input: Vec<WareAmount>,
    pub output: Vec<WareAmount>,
    pub time: DeltaTime,
}

impl Receipt {
    pub fn request_wares_id(&self) -> Vec<WareId> {
        self.input.iter().map(|i| i.ware_id).collect()
    }

    pub fn provide_wares_id(&self) -> Vec<WareId> {
        self.output.iter().map(|i| i.ware_id).collect()
    }
}

#[derive(Debug, Clone, Component)]
pub struct Factory {
    pub production: Receipt,
    pub production_time: Option<TotalTime>,
}

impl Factory {
    pub fn new(production: Receipt) -> Self {
        Factory {
            production,
            production_time: None,
        }
    }

    pub fn get_cargos_allocation(&self) -> Vec<WareId> {
        let mut result = Vec::new();
        result.extend(self.production.input.iter().map(|i| i.ware_id));
        result.extend(self.production.output.iter().map(|i| i.ware_id));
        result
    }
}

// pub struct FactorySystem;
//
// impl<'a> System<'a> for FactorySystem {
//     type SystemData = (
//         Read<'a, TotalTime>,
//         Entities<'a>,
//         WriteStorage<'a, Cargo>,
//         WriteStorage<'a, Factory>,
//     );
//
//     fn run(&mut self, data: Self::SystemData) {
//         log::trace!("running");
//
//         let (total_time, entities, mut cargos, mut factories) = data;
//
//         let total_time = *total_time;
//
//         for (entity, cargo, factory) in (&*entities, &mut cargos, &mut factories).join() {
//             match factory.production_time {
//                 Some(time) if total_time.is_after(time) => {
//                     // production ready
//                     match cargo.add_all_or_none(&factory.production.output) {
//                         Ok(()) => {
//                             log::debug!(
//                                 "{:?} factory complete production, adding cargo: {:?}",
//                                 entity,
//                                 &factory.production.output,
//                             );
//                             factory.production_time = None;
//                         }
//                         Err(err) => {
//                             log::warn!(
//                                 "{:?} factory complete production, but fail to add cargo by {:?}",
//                                 entity,
//                                 err
//                             );
//                         }
//                     }
//                 }
//
//                 Some(_time) => {
//                     // producing
//                     log::trace!("{:?} factory producing", entity);
//                 }
//
//                 None => {
//                     // check if have enough cargo to start a new production
//                     match cargo.remove_all_or_none(&factory.production.input) {
//                         Ok(()) => {
//                             let end_time = total_time.add(factory.production.time);
//                             log::trace!(
//                                 "{entity:?} factory start production, ends at {end_time:?}"
//                             );
//                             factory.production_time = Some(end_time);
//                         }
//                         Err(err) => {
//                             log::trace!(
//                                 "{entity:?} factory fail to remove cargo by {err:?}, skipping"
//                             );
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::game::wares::Volume;
//
//     use crate::test::test_system;
//
//     const REQUIRE_ORE: Volume = 10;
//     const REQUIRE_ENERGY: Volume = 100;
//     const PRODUCTION_TIME: f32 = 5.0;
//     const TOTAL_CARGO: Volume = 200;
//     const PRODUCED_PLATE: Volume = 10;
//
//     #[test]
//     fn test_factory_system_should_not_start_production_without_enough_cargo() {
//         run_factory(0, 0, 3.0, None, None, 0, 0, 0);
//     }
//
//     #[test]
//     fn test_factory_system_should_not_start_production_without_both_enough_cargo() {
//         let ore_amount = REQUIRE_ORE - 5;
//         run_factory(ore_amount, 0, 3.0, None, None, ore_amount, 0, 0);
//     }
//
//     #[test]
//     fn test_factory_system_should_start_production_with_both_cargo() {
//         let current_time = 3.0;
//         run_factory(
//             REQUIRE_ORE,
//             REQUIRE_ENERGY,
//             current_time,
//             None,
//             Some(current_time + PRODUCTION_TIME as f64),
//             0,
//             10,
//             0,
//         );
//     }
//
//     #[test]
//     fn test_factory_system_should_keep_producing() {
//         run_factory(
//             REQUIRE_ORE,
//             REQUIRE_ENERGY,
//             3.0,
//             Some(8.0),
//             Some(8.0),
//             REQUIRE_ORE,
//             REQUIRE_ENERGY,
//             0,
//         );
//     }
//
//     #[test]
//     fn test_factory_system_should_produce() {
//         run_factory(0, 0, 9.0, Some(8.0), None, 0, 0, PRODUCED_PLATE);
//     }
//
//     #[test]
//     fn test_factory_system_should_not_complete_production_if_cargo_is_full() {
//         run_factory(TOTAL_CARGO, 0, 9.0, Some(8.0), Some(8.0), 0, 0, 0);
//     }
//
//     fn run_factory(
//         ore_volume: Volume,
//         energy_volume: Volume,
//         total_time: f64,
//         production_time: Option<f64>,
//         expect_produce_at: Option<f64>,
//         _expected_ore: Volume,
//         _expected_energy: Volume,
//         expected_plates: Volume,
//     ) {
//         let (world, (entity, plate_id)) = test_system(FactorySystem, move |world| {
//             let ore_id = world.spawn_empty().id();
//             let energy_id = world.spawn_empty().id();
//             let plate_id = world.spawn_empty().id();
//
//             let production = Receipt {
//                 label: "ore processing".to_string(),
//                 input: vec![
//                     WareAmount::new(ore_id, REQUIRE_ORE),
//                     WareAmount::new(energy_id, REQUIRE_ENERGY),
//                 ],
//                 output: vec![WareAmount::new(plate_id, PRODUCED_PLATE)],
//                 time: DeltaTime(PRODUCTION_TIME),
//             };
//
//             let mut cargo = Cargo::new(TOTAL_CARGO);
//             cargo.add(ore_id, ore_volume).expect("fail to add ore");
//             cargo
//                 .add(energy_id, energy_volume)
//                 .expect("fail to add energy");
//
//             world.insert(TotalTime(total_time));
//
//             let entity = world
//                 .create_entity()
//                 .insert(cargo)
//                 .insert(Factory {
//                     production,
//                     production_time: production_time.map(|time| TotalTime(time)),
//                 })
//                 .id();
//
//             (entity, plate_id)
//         });
//
//         let cargo = world.read_storage::<Cargo>().get(entity).unwrap().clone();
//         assert_eq!(cargo.get_amount(plate_id), expected_plates);
//
//         let factory = world.read_storage::<Factory>().get(entity).unwrap().clone();
//         assert_eq!(
//             factory.production_time.map(|i| i.as_f64()),
//             expect_produce_at
//         );
//     }
// }
