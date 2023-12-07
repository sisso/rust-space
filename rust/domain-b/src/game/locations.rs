// mod index_per_sector_system;

use bevy_ecs::prelude::*;
use commons::math::{Distance, Rad, P2};
use std::collections::HashMap;

use super::objects::*;
use super::sectors::*;
use crate::game::utils::*;

// use crate::game::locations::index_per_sector_system::*;
use crate::game::{GameInitContext, RequireInitializer};

#[derive(Debug, Clone, Copy, Component)]
pub struct LocationSpace {
    pub pos: P2,
    pub sector_id: SectorId,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct LocationOrbit {
    pub parent_id: Entity,
    pub distance: Distance,
    pub start_time: TotalTime,
    pub start_angle: Rad,
    pub speed: Speed,
}

// TODO: move to orbits
impl LocationOrbit {
    pub fn new(target_id: ObjId) -> Self {
        LocationOrbit {
            parent_id: target_id,
            distance: 0.0,
            start_time: Default::default(),
            start_angle: 0.0,
            speed: Speed(0.0),
        }
    }
}

#[derive(Clone, Debug, Copy, Component)]
pub struct LocationDocked {
    pub parent_id: ObjId,
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
        todo!()
        // context
        //     .dispatcher
        //     .add(IndexPerSectorSystem, INDEX_SECTOR_SYSTEM, &[]);
    }
}

impl Locations {
    pub fn new() -> Self {
        Locations {}
    }

    pub fn is_near(loc_a: &LocationSpace, loc_b: &LocationSpace) -> bool {
        loc_a.sector_id == loc_b.sector_id
            && loc_a.pos.distance_squared(loc_b.pos) < MIN_DISTANCE_SQR
    }

    pub fn is_near_maybe(pos_a: Option<&LocationSpace>, pos_b: Option<&LocationSpace>) -> bool {
        match (pos_a, pos_b) {
            (Some(pos_a), Some(pos_b)) => Locations::is_near(pos_a, pos_b),
            _ => false,
        }
    }

    pub fn is_near_from_storage(world: &World, obj_a: ObjId, obj_b: ObjId) -> bool {
        Locations::is_near_maybe(
            world.get::<LocationSpace>(obj_a),
            world.get::<LocationSpace>(obj_b),
        )
    }

    /// same as resolve_space_position, but receive a world and the storages are fetched from it
    pub fn resolve_space_position(
        obj_id: ObjId,
        query: &Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    ) -> Option<LocationSpace> {
        match query.get(obj_id).ok()? {
            (_, Some(space), _) => Some(space.clone()),
            (_, _, Some(docked)) => Self::resolve_space_position(docked.parent_id, query),
            _ => None,
        }
    }

    // /// recursive search through docked entities until find what space position entity is at
    // pub fn resolve_space_position<'a, D1, D2>(
    //     world: &World,
    //     locations_space: &Storage<'a, LocationSpace, D1>,
    //     locations_docked: &Storage<'a, LocationDocked, D2>,
    //     obj_id: ObjId,
    // ) -> Option<LocationSpace>
    // where
    //     D1: Deref<Target = MaskedStorage<LocationSpace>>,
    //     D2: Deref<Target = MaskedStorage<LocationDocked>>,
    // {
    //     match (locations_space.get(obj_id), locations_docked.get(obj_id)) {
    //         (Some(at_space), _) => Some(at_space.clone()),
    //         (_, Some(docked)) => {
    //             Self::resolve_space_position(locations_space, locations_docked, docked.parent_id)
    //         }
    //         _ => None,
    //     }
    // }

    // pub fn is_docked_at<D1>(
    //     locations_docked: &Storage<LocationDocked, D1>,
    //     obj_id: ObjId,
    //     target_id: ObjId,
    // ) -> bool
    // where
    //     D1: Deref<Target = MaskedStorage<LocationDocked>>,
    // {
    //     locations_docked
    //         .get(obj_id)
    //         .map(|docked| docked.parent_id == target_id)
    //         .unwrap_or(false)
    // }
}

pub fn update_locations_index(world: &World) {
    todo!()
    // let mut system = index_per_sector_system::IndexPerSectorSystem {};
    // system.run_now(world);
}
