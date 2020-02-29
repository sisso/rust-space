use specs::prelude::*;
use crate::utils::{DeltaTime, TotalTime};
use std::collections::HashMap;
use crate::game::wares::{WareId, Cargo};
use crate::game::{RequireInitializer, GameInitContext};

#[derive(Debug, Clone)]
pub struct Production {
    pub input: HashMap<WareId, f32>,
    pub output: HashMap<WareId, f32>,
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

        for (entity, cargo, shipyard) in (&*entities, &mut cargos, &mut factories).join() {

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
    fn test_factory_system_should_not_start_production_without_enough_cargo() {}
}
