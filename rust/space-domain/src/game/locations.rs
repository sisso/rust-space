mod index_per_sector_system;

use specs::prelude::*;
use std::collections::HashMap;

use super::objects::*;
use super::sectors::*;
use crate::utils::*;

use crate::game::jsons::JsonValueExtra;
use crate::game::locations::index_per_sector_system::*;

#[derive(Debug, Clone, Component)]
pub struct LocationSpace {
    pub pos: Position
}

#[derive(Debug, Clone, Component)]
pub struct LocationDock {
    pub docked_id: ObjId
}

#[derive(Debug, Clone, Component)]
pub struct LocationSector {
    pub sector_id: SectorId
}

#[derive(Debug, Clone, Component)]
pub struct Moveable {
    pub speed: Speed
}

#[derive(Clone,Debug,Default,Component)]
pub struct EntityPerSectorIndex {
    pub index: HashMap<SectorId, Vec<ObjId>>,
    pub index_extractables: HashMap<SectorId, Vec<ObjId>>,
}

impl EntityPerSectorIndex {
    pub fn new() -> Self {
        EntityPerSectorIndex {
            index: Default::default(),
            index_extractables: Default::default(),
        }
    }

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

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        world.register::<Moveable>();
        dispatcher.add(IndexPerSectorSystem, "index_by_sector", &[]);
    }

    pub fn execute(&mut self, world: &mut World) {

    }
}
