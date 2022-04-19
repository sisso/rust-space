use crate::game::{GameInitContext, RequireInitializer};
use specs::prelude::*;

#[derive(Clone, Debug, Component)]
pub struct Fleet {}

impl RequireInitializer for Fleet {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Fleet>();
    }
}
