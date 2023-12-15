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
