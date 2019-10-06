use std::collections::HashMap;

use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage};

use super::objects::*;
use super::sectors::*;
use crate::utils::*;

use crate::game::save::{Save, Load, CanSave, CanLoad};
use crate::game::jsons::JsonValueExtra;

#[derive(Clone, Debug)]
pub struct LocationSpace {
    pub sector_id: SectorId,
    pub pos: Position
}

#[derive(Clone, Debug)]
pub struct LocationDock {
    pub docked_id: ObjId
}

#[derive(Clone, Debug)]
pub struct Moveable {
    pub speed: Speed
}

impl SpecComponent for LocationSpace {
    type Storage = VecStorage<Self>;
}

impl SpecComponent for LocationDock {
    type Storage = DenseVecStorage<Self>;
}

impl SpecComponent for Moveable {
    type Storage = DenseVecStorage<Self>;
}

pub struct Locations {
}

impl Locations {
    pub fn new() -> Self {
        Locations {
        }
    }

    pub fn init_world(world: &mut World) {
        world.register::<LocationSpace>();
        world.register::<LocationDock>();
        world.register::<Moveable>();
    }
}
