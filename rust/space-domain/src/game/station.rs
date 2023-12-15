use bevy_ecs::prelude::*;

#[derive(Clone, Debug, Component)]
pub struct Station {}

impl Station {
    pub fn new() -> Self {
        Station {}
    }
}

pub struct Stations;
