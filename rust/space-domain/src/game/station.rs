use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Station {}

impl Station {
    pub fn new() -> Self {
        Station {}
    }
}

pub struct Stations;
