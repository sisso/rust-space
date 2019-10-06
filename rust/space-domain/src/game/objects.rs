use std::collections::HashMap;

use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage};

use crate::utils::*;

pub type ObjId = Entity;

#[derive(Debug,Copy,Clone)]
pub struct HasDock;

impl SpecComponent for HasDock {
    type Storage = HashMapStorage<Self>;
}

pub fn init_world(world: &mut World) {
    world.register::<HasDock>();
}

pub struct Objects;

impl Objects {
    pub fn new() -> Self {
        Objects {}
    }
}
