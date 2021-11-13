use crate::game::wares::{Cargo, WareAmount, WareId};
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::{DeltaTime, TotalTime};
use specs::prelude::*;


#[derive(Debug, Clone)]
pub struct Receipt {
    pub input: Vec<WareAmount>,
    pub output: Vec<WareAmount>,
    pub time: DeltaTime,
}

impl Receipt {
    pub fn request_wares_id(&self) -> Vec<WareId> {
        self.input
            .iter()
            .map(|WareAmount(ware_id, _)| *ware_id)
            .collect()
    }

    pub fn provide_wares_id(&self) -> Vec<WareId> {
        self.output
            .iter()
            .map(|WareAmount(ware_id, _)| *ware_id)
            .collect()
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

    pub fn setup_cargo(&self, cargo: &mut Cargo) {
        let mut wares = vec![];
        wares.extend(self.production.input.iter().map(|i| i.0));
        wares.extend(self.production.output.iter().map(|i| i.0));
        cargo.set_whitelist(wares);
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
        log::trace!("running");

        let (total_time, entities, mut cargos, mut factories) = data;

        let total_time = *total_time;

        for (entity, cargo, factory) in (&*entities, &mut cargos, &mut factories).join() {
            match factory.production_time {
                Some(time) if total_time.is_after(time) => {
                    // production ready
                    match cargo.add_all(&factory.production.output) {
                        Ok(()) => {
                            log::debug!(
                                "{:?} adding production to cargo: {:?}",
                                entity,
                                &factory.production.output,
                            );
                            factory.production_time = None;
                        }
                        _ => {}
                    }
                }

                Some(_time) => {
                    // producing
                }

                None => {
                    // check if have enough cargo to start a new production
                    match cargo.remove_all(&factory.production.input) {
                        Ok(()) => {
                            factory.production_time = Some(total_time.add(factory.production.time));
                        }
                        _ => {}
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

    const REQUIRE_ORE: f32 = 1.0;
    const REQUIRE_ENERGY: f32 = 10.0;
    const PRODUCTION_TIME: f32 = 5.0;
    const TOTAL_CARGO: f32 = 20.0;
    const PRODUCED_PLATE: f32 = 1.0;

    #[test]
    fn test_factory_system_should_not_start_production_without_enough_cargo() {
        run_factory(0.0, 0.0, 3.0, None, None, 0.0, 0.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_not_start_production_without_both_enough_cargo() {
        let ore_amount = REQUIRE_ORE - 0.5;
        run_factory(ore_amount, 0.0, 3.0, None, None, ore_amount, 0.0, 0.0);
    }

    #[test]
    fn test_factory_system_should_start_production_with_both_cargo() {
        let current_time = 3.0;
        run_factory(
            REQUIRE_ORE,
            REQUIRE_ENERGY,
            current_time,
            None,
            Some(current_time + PRODUCTION_TIME as f64),
            0.0,
            1.0,
            0.0,
        );
    }

    #[test]
    fn test_factory_system_should_keep_producing() {
        run_factory(
            REQUIRE_ORE,
            REQUIRE_ENERGY,
            3.0,
            Some(8.0),
            Some(8.0),
            REQUIRE_ORE,
            REQUIRE_ENERGY,
            0.0,
        );
    }

    #[test]
    fn test_factory_system_should_produce() {
        run_factory(0.0, 0.0, 9.0, Some(8.0), None, 0.0, 0.0, PRODUCED_PLATE);
    }

    #[test]
    fn test_factory_system_should_not_complete_production_if_cargo_is_full() {
        run_factory(TOTAL_CARGO, 0.0, 9.0, Some(8.0), Some(8.0), 0.0, 0.0, 0.0);
    }

    fn run_factory(
        ore: f32,
        energy: f32,
        total_time: f64,
        production_time: Option<f64>,
        expect_produce_at: Option<f64>,
        _expected_ore: f32,
        _expected_energy: f32,
        expected_plates: f32,
    ) {
        let (world, (entity, plate_id)) = test_system(FactorySystem, move |world| {
            let ore_id = world.create_entity().build();
            let energy_id = world.create_entity().build();
            let plate_id = world.create_entity().build();

            let production = Receipt {
                input: vec![
                    WareAmount(ore_id, REQUIRE_ORE),
                    WareAmount(energy_id, REQUIRE_ENERGY),
                ],
                output: vec![WareAmount(plate_id, PRODUCED_PLATE)],
                time: DeltaTime(PRODUCTION_TIME),
            };

            let mut cargo = Cargo::new(TOTAL_CARGO);
            cargo.add(ore_id, ore).unwrap();
            cargo.add(energy_id, energy).unwrap();

            world.insert(TotalTime(total_time));

            let entity = world
                .create_entity()
                .with(cargo)
                .with(Factory {
                    production,
                    production_time: production_time.map(|time| TotalTime(time)),
                })
                .build();

            (entity, plate_id)
        });

        let cargo = world.read_storage::<Cargo>().get(entity).unwrap().clone();
        assert_eq!(cargo.get_amount(plate_id), expected_plates);

        let factory = world.read_storage::<Factory>().get(entity).unwrap().clone();
        assert_eq!(
            factory.production_time.map(|i| i.as_f64()),
            expect_produce_at
        );
    }
}
