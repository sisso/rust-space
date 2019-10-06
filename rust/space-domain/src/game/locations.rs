mod index_per_sector_system;

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

#[derive(Clone,Debug,Default)]
pub struct EntityPerSectorIndex {
    pub index: HashMap<SectorId, Vec<ObjId>>,
    pub index_extractables: HashMap<SectorId, Vec<ObjId>>,
}

impl EntityPerSectorIndex {
    pub fn clear(&mut self) {
        self.index.clear();
        self.index_extractables.clear();
    }

    pub fn add(&mut self, sector_id: SectorId, obj_id: ObjId) {
        self.index.entry(sector_id)
            .and_modify(|list| list.push(obj_id))
            .or_insert(vec![obj_id]);
    }

    pub fn add_extractable(&mut self, sector_id: SectorId, obj_id: ObjId) {
        self.index_extractables.entry(sector_id)
            .and_modify(|list| list.push(obj_id))
            .or_insert(vec![obj_id]);
    }
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

    pub fn execute(&mut self, world: &mut World) {

    }
}
