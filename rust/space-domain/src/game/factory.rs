use std::collections::HashMap;

use super::objects::{ObjId};
use crate::utils::*;
use crate::game::wares::*;

#[derive(Clone, Debug)]
pub struct Production {
    pub input: HashMap<WareId, f32>,
    pub output: HashMap<WareId, f32>,
    pub time: DeltaTime,
}

#[derive(Clone, Debug)]
struct ProductionState {
    pub complete_time: Option<TotalTime>,
}

impl ProductionState {
    pub fn new() -> Self {
        ProductionState { complete_time: None }
    }
}

#[derive(Clone, Debug)]
pub struct Factory {
    production: Vec<(Production, ProductionState)>,
}

impl Factory {
    pub fn new(production: Vec<Production>) -> Self {
        Factory {
            production: production.into_iter().map(|production| {
                (production, ProductionState::new())
            }).collect()
        }
    }

    pub fn get_count(&self) -> usize {
        self.production.len()
    }

    pub fn get_percentage(&self, index: usize, time: TotalTime) -> f32 {
        self.production.get(index).and_then(|(production, state)| {
            state.complete_time.map(|complete_time| {
                let require_time = complete_time.sub(time);

                if require_time.as_f32() <= 0.0 {
                    0.0
                } else {
                    1.0 - require_time.as_f32() / production.time.as_f32()
                }
            })
        }).unwrap_or(0.0)
    }

    pub fn update(&mut self, time: TotalTime, cargo: &mut Cargo) {
        for (production, state) in &mut self.production {
            match state.complete_time {
                Some(end_time) if time.is_after(end_time) => {
                    if Factory::complete_production(cargo, production, state).is_ok() {
                        Factory::try_start_production(time, cargo, production, state);
                    }
                },
                Some(_) => {},
                None => {
                    Factory::try_start_production(time, cargo, production, state);
                }
            }
        }
    }

    pub fn is_producing(&self, index: usize) -> bool {
        let (_, state) = self.production.get(index).unwrap();
        state.complete_time.is_some()
    }

    fn complete_production(cargo: &mut Cargo, production: &mut Production, state: &mut ProductionState) -> Result<(),()> {
        let total_to_add = production.output.iter().map(|(_, amount)| amount).sum();
        if cargo.free_space() < total_to_add {
            // not enough space to complete production
            return Err(());
        }

        for (ware_id, amount) in &production.output {
            cargo.add(*ware_id, *amount).expect("not able to add produced cargo, it is full")
        }

        state.complete_time = None;
        Ok(())
    }

    fn try_start_production(time: TotalTime, cargo: &mut Cargo, production: &mut Production, state: &mut ProductionState) {
        let has_enough_input = production.input.iter().all(|(ware_id, require_amount)| {
            cargo.get_amount(*ware_id) >= *require_amount
        });

        if !has_enough_input {
            return;
        }

        for (ware_id, amount) in &production.input {
            cargo.remove(*ware_id, *amount).expect("failed to remove cargo");
        }

        state.complete_time = Some(time.add(production.time))
    }
}

#[derive(Clone, Debug)]
struct State {
    factory: Factory
}

impl State {
    pub fn new(factory: Factory) -> Self {
        State {
            factory
        }
    }
}

pub struct Factories {
    index: HashMap<ObjId, State>,
}

impl Factories {
    pub fn new() -> Self {
        Factories {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, obj_id: ObjId, factory: Factory) {
        self.index.insert(obj_id, State::new(factory));
    }

    pub fn set(&mut self, obj_id: ObjId, value: Factory) {
        let mut state = self.index.get_mut(&obj_id).unwrap();
        info!("Factories", &format!("set {:?}: {:?}", obj_id, value));
        state.factory = value;
    }

    pub fn get(&self, id: ObjId) -> &Factory {
        let state = self.index.get(&id).unwrap();
        &state.factory
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::wares::*;

    const WARE_ORE: WareId = WareId(0);
    const WARE_POWER: WareId = WareId(1);
    const WARE_IRON: WareId = WareId(2);
    const WARE_COPPER: WareId = WareId(3);
    const WARE_PLATE: WareId = WareId(4);

    #[test]
    fn test_factory_production_acceptance() {
        let ore_production = Production {
            input: vec![(WARE_ORE, 1.0), (WARE_POWER, 2.0)].into_iter().collect(),
            output: vec![(WARE_IRON, 1.0), (WARE_COPPER, 0.25)].into_iter().collect(),
            time: DeltaTime(2.0),
        };

        let plate_production = Production {
            input: vec![(WARE_IRON, 1.0)].into_iter().collect(),
            output: vec![(WARE_PLATE, 2.0)].into_iter().collect(),
            time: DeltaTime(1.0),
        };
        
        let mut factory = Factory::new(vec![ore_production, plate_production]);
        let mut cargo = Cargo::new(100.0);

        assert_eq!(2, factory.get_count());

        // no cargo, nothing happens
        factory.update(TotalTime(1.0), &mut cargo);
        assert!(!factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(0.0, cargo.get_current());

        // add one input, nothings happens
        cargo.add(WARE_ORE, 10.0);
        factory.update(TotalTime(2.0), &mut cargo);
        assert!(!factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(10.0, cargo.get_current());

        // add less that require input, nothings happens
        cargo.add(WARE_POWER, 1.5);
        factory.update(TotalTime(3.0), &mut cargo);
        assert!(!factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(11.5, cargo.get_current());

        // add enough input, inputs are removed and production starts
        let total_time = TotalTime(4.0);

        cargo.add(WARE_POWER, 0.5);
        factory.update(total_time, &mut cargo);
        assert!(factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(9.0, cargo.get_current());

        // check percentages
        assert_eq!(0.5, factory.get_percentage(0, TotalTime(5.0)));
        assert_eq!(0.75, factory.get_percentage(0, TotalTime(5.5)));
        assert_eq!(0.0, factory.get_percentage(1, TotalTime(5.0)));

        // still producing
        let total_time = TotalTime(5.5);
        factory.update(total_time, &mut cargo);
        assert!(factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(9.0, cargo.get_current());

        // complete production and chain next one
        let total_time = TotalTime(6.0);
        factory.update(total_time, &mut cargo);
        // produce 1 iron and 0.25 copper, iron is used to start next line production
        assert!(!factory.is_producing(0));
        assert!(factory.is_producing(1));
        assert_eq!(9.25, cargo.get_current());
        assert_eq!(0.0, factory.get_percentage(0, total_time));
        assert_eq!(90, (factory.get_percentage(1, TotalTime(6.9)) * 100.0) as i32);

        // complete both productions
        let total_time = TotalTime(7.0);
        factory.update(total_time, &mut cargo);
        assert!(!factory.is_producing(0));
        assert!(!factory.is_producing(1));
        assert_eq!(11.25, cargo.get_current());
        assert_eq!(0.0, factory.get_percentage(0, total_time));
        assert_eq!(0.0, factory.get_percentage(1, total_time));
    }

    #[test]
    fn test_factory_production_continuous_production() {
        let ore_production = Production {
            input: vec![(WARE_ORE, 1.0)].into_iter().collect(),
            output: vec![(WARE_IRON, 1.0)].into_iter().collect(),
            time: DeltaTime(1.0),
        };

        let mut factory = Factory::new(vec![ore_production]);
        let mut cargo = Cargo::new(10.0);
        cargo.add(WARE_ORE, 10.0);

        factory.update(TotalTime(0.0), &mut cargo);
        assert_eq!(factory.is_producing(0), true);
        assert_eq!(cargo.get_amount(WARE_ORE), 9.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 0.0);

        factory.update(TotalTime(1.0), &mut cargo);
        assert_eq!(factory.is_producing(0), true);
        assert_eq!(cargo.get_amount(WARE_ORE), 8.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 1.0);

        for time in 2..9 {
            factory.update(TotalTime(time as f64), &mut cargo);
            assert_eq!(factory.is_producing(0), true);
            assert_eq!(cargo.get_amount(WARE_ORE), 9.0 - time as f32);
            assert_eq!(cargo.get_amount(WARE_IRON), time as f32);
        }

        factory.update(TotalTime(10.0), &mut cargo);
        assert_eq!(factory.is_producing(0), true);
        assert_eq!(cargo.get_amount(WARE_ORE), 0.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 9.0);

        factory.update(TotalTime(11.0), &mut cargo);
        assert_eq!(factory.is_producing(0), false);
        assert_eq!(cargo.get_amount(WARE_ORE), 0.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 10.0);
    }

    #[test]
    fn test_factory_should_stop_if_can_not_unload_production() {
        let ore_production = Production {
            input: vec![(WARE_ORE, 1.0)].into_iter().collect(),
            output: vec![(WARE_IRON, 3.0)].into_iter().collect(),
            time: DeltaTime(1.0),
        };

        let mut factory = Factory::new(vec![ore_production]);
        let mut cargo = Cargo::new(5.0);
        cargo.add(WARE_ORE, 5.0);

        factory.update(TotalTime(0.0), &mut cargo);
        assert_eq!(factory.is_producing(0), true);
        assert_eq!(cargo.get_amount(WARE_ORE), 4.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 0.0);

        // production should be complete, but get stuck by not enough space in cargo
        for time in 1..5 {
            factory.update(TotalTime(time as f64), &mut cargo);
            assert_eq!(factory.is_producing(0), true);
            assert_eq!(cargo.get_amount(WARE_ORE), 4.0);
            assert_eq!(cargo.get_amount(WARE_IRON), 0.0);
        }

        // free space to enable production to complete
        cargo.remove(WARE_ORE, 4.0);
        factory.update(TotalTime(5.1), &mut cargo);
        assert_eq!(factory.is_producing(0), false);
        assert_eq!(cargo.get_amount(WARE_ORE), 0.0);
        assert_eq!(cargo.get_amount(WARE_IRON), 3.0);
    }
}
