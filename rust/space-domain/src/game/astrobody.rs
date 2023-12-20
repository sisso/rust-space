use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AstroBodyKind {
    Star,
    Planet,
}

#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct AstroBody {
    pub kind: AstroBodyKind,
}

pub struct AstroBodies;
