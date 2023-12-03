use crate::game::{GameInitContext, RequireInitializer};
use bevy_ecs::prelude::*;

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
