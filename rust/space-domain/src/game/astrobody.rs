use crate::game::{GameInitContext, RequireInitializer};
use specs::prelude::*;

#[derive(Clone, Debug, Component)]
pub enum AstroBody {
    Star,
    Planet,
}

pub struct AstroBodies;

impl RequireInitializer for AstroBodies {
    fn init(context: &mut GameInitContext) {
        context.world.register::<AstroBody>();
    }
}
