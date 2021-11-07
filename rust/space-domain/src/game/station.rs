use crate::game::{GameInitContext, RequireInitializer};
use specs::prelude::*;

// TODO: not in used
/// Static object that usually contains docks, storage, factories and shipyards
#[derive(Clone, Debug, Component)]
pub struct Station {}

impl Station {
    pub fn new() -> Self {
        Station {}
    }
}

pub struct Stations;

impl RequireInitializer for Stations {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Station>();
    }
}
