mod index_per_sector_system;

use specs::prelude::*;
use std::collections::HashMap;

use super::objects::*;
use super::sectors::*;
use crate::utils::*;

use crate::game::jsons::JsonValueExtra;
use crate::game::locations::index_per_sector_system::*;
use crate::game::{GameInitContext, RequireInitializer};
use shred::Fetch;
use specs::storage::MaskedStorage;
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct LocationSpace {
    pub pos: Position,
    pub sector_id: SectorId,
}

#[derive(Debug, Clone, Component)]
pub enum Location {
    Space { pos: Position, sector_id: SectorId },
    Dock { docked_id: ObjId },
}

impl Location {
    pub fn as_space(&self) -> Option<LocationSpace> {
        match self {
            Location::Space { pos, sector_id } => Some(LocationSpace {
                pos: *pos,
                sector_id: *sector_id,
            }),
            _ => None,
        }
    }

    /// Utility method since we can not easily reference a enum type
    pub fn set_pos(&mut self, new_pos: Position) -> Result<(), ()> {
        match self {
            Location::Space { pos, .. } => {
                *pos = new_pos;
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// Utility method since we can not easily reference a enum type
    pub fn get_pos(&self) -> Option<Position> {
        match self {
            Location::Space { pos, .. } => Some(pos.clone()),
            _ => None,
        }
    }

    /// Utility method since we can not easily reference a enum type
    pub fn get_sector_id(&self) -> Option<SectorId> {
        match self {
            Location::Space { sector_id, .. } => Some(sector_id.clone()),
            _ => None,
        }
    }

    /// Utility method since we can not easily reference a enum type
    pub fn as_docked(&self) -> Option<ObjId> {
        match self {
            Location::Dock { docked_id } => Some(docked_id.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Moveable {
    pub speed: Speed,
}

pub trait SectorDistanceIndex {
    fn distance(&self, a: SectorId, b: SectorId) -> u32;
}

// TODO: make more flexible, like tags?
#[derive(Clone, Debug, Default, Component)]
pub struct EntityPerSectorIndex {
    pub index: HashMap<SectorId, Vec<ObjId>>,
    pub index_extractables: HashMap<SectorId, Vec<ObjId>>,
    pub index_stations: HashMap<SectorId, Vec<ObjId>>,
}

impl EntityPerSectorIndex {
    pub fn new() -> Self {
        EntityPerSectorIndex {
            index: Default::default(),
            index_extractables: Default::default(),
            index_stations: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.index_extractables.clear();
    }

    pub fn add(&mut self, sector_id: SectorId, obj_id: ObjId) {
        self.index
            .entry(sector_id)
            .and_modify(|list| list.push(obj_id))
            .or_insert(vec![obj_id]);
    }

    pub fn add_extractable(&mut self, sector_id: SectorId, obj_id: ObjId) {
        self.index_extractables
            .entry(sector_id)
            .and_modify(|list| list.push(obj_id))
            .or_insert(vec![obj_id]);
    }

    pub fn add_stations(&mut self, sector_id: SectorId, obj_id: ObjId) {
        self.index_stations
            .entry(sector_id)
            .and_modify(|list| list.push(obj_id))
            .or_insert(vec![obj_id]);
    }

    // TODO: return properly distance, not only 0 or 1
    /// returns the sector_id, distance, object_id
    pub fn search_nearest_extractable<'a>(
        &'a self,
        from_sector_id: SectorId,
    ) -> impl Iterator<Item = (SectorId, u32, ObjId)> + 'a {
        self.index_extractables
            .iter()
            .flat_map(move |(&sector_id, list)| {
                // TODO: remove the collect
                list.iter()
                    .map(|id| {
                        let same_sector = sector_id == from_sector_id;
                        let distance = if same_sector { 0 } else { 1 };
                        (sector_id, distance, *id)
                    })
                    .collect::<Vec<(SectorId, u32, ObjId)>>()
            })
    }

    // TODO: should be a iterator from nearest to far
    pub fn search_nearest_stations<'a>(
        &'a self,
        from_sector_id: SectorId,
    ) -> impl Iterator<Item = (SectorId, u32, ObjId)> + 'a {
        self.index_stations
            .iter()
            .flat_map(move |(&sector_id, list)| {
                list.iter()
                    .map(|id| {
                        let same_sector = sector_id == from_sector_id;
                        let distance = if same_sector { 0 } else { 1 };
                        (sector_id, distance, *id)
                    })
                    .collect::<Vec<(SectorId, u32, ObjId)>>()
            })
    }
}

pub const INDEX_SECTOR_SYSTEM: &str = "index_sector";

pub struct Locations {}

impl RequireInitializer for Locations {
    fn init(context: &mut GameInitContext) {
        context
            .dispatcher
            .add(IndexPerSectorSystem, INDEX_SECTOR_SYSTEM, &[]);
    }
}

impl Locations {
    pub fn new() -> Self {
        Locations {}
    }

    pub fn is_near(loc_a: &Location, loc_b: &Location) -> bool {
        match (loc_a, loc_b) {
            (
                Location::Space {
                    pos: pos_a,
                    sector_id: sector_id_a,
                },
                Location::Space {
                    pos: pos_b,
                    sector_id: sector_id_b,
                },
            ) => sector_id_a == sector_id_b && V2::distance(&pos_a, &pos_b) < MIN_DISTANCE,
            _ => false,
        }
    }

    pub fn is_near_maybe(pos_a: Option<&Location>, pos_b: Option<&Location>) -> bool {
        match (pos_a, pos_b) {
            (Some(pos_a), Some(pos_b)) => Locations::is_near(pos_a, pos_b),
            _ => false,
        }
    }

    pub fn is_near_from_storage(
        locations: &ReadStorage<Location>,
        obj_a: ObjId,
        obj_b: ObjId,
    ) -> bool {
        let pos_a = locations.get(obj_a);
        let pos_b = locations.get(obj_b);
        Locations::is_near_maybe(pos_a, pos_b)
    }

    /// recursive search through docked entities until find what space position entity is
    pub fn resolve_space_position(
        locations: &ReadStorage<Location>,
        obj: ObjId,
    ) -> Option<LocationSpace> {
        match locations.get(obj) {
            Some(location @ Location::Space { .. }) => location.as_space(),
            Some(Location::Dock { docked_id }) => {
                Locations::resolve_space_position(locations, *docked_id)
            }
            _ => None,
        }
    }

    /// recursive search for position, but receive an already fettched one
    pub fn resolve_space_position_from(
        locations: &ReadStorage<Location>,
        location: &Location,
    ) -> Option<LocationSpace> {
        match location {
            location @ Location::Space { .. } => location.as_space(),
            Location::Dock { docked_id } => {
                Locations::resolve_space_position(locations, *docked_id)
            }
        }
    }
}
