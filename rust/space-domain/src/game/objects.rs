use crate::utils::*;
use specs::prelude::*;
use std::collections::HashMap;

pub type ObjId = Entity;

// TODO: merge with station
#[derive(Debug, Copy, Clone, Component)]
pub struct HasDock;

pub struct Objects;

impl Objects {
    pub fn new() -> Self {
        Objects {}
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        world.register::<HasDock>();
    }
}
