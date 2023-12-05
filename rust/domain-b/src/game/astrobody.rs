use crate::game::{GameInitContext, RequireInitializer};
use bevy_ecs::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AstroBodyKind {
    Star,
    Planet,
}

#[derive(Clone, Debug, Component)]
pub struct AstroBody {
    pub kind: AstroBodyKind,
}

pub struct AstroBodies;

impl RequireInitializer for AstroBodies {
    fn init(context: &mut GameInitContext) {}
}
