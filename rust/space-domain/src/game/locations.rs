use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use commons::math::{Distance, Rad, P2};
use std::collections::HashMap;

use super::objects::*;
use super::sectors::*;
use crate::game::utils::*;

use crate::game::dock::HasDocking;
use crate::game::extractables::Extractable;

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

/// Index entities to provide fast look up. This system is update on end of tick, so is expected
/// to provide outdated data during a run.
/// - what ships are in sector 0?
/// - what is nearest asteroid from sector 2?
/// - tags?
/// - space partition?
/// - collision prediction?
#[derive(Clone, Debug, Default, Resource)]
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

pub struct Locations {}

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

    pub fn get_location_space(world: &World, obj_id: ObjId) -> Option<LocationSpace> {
        world.get::<LocationSpace>(obj_id).cloned()
    }

    pub fn resolve_space_position_system(
        In(obj_id): In<ObjId>,
        query: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    ) -> Option<LocationSpace> {
        Self::resolve_space_position(&query, obj_id)
    }

    /// same as resolve_space_position, but receive a world and the storages are fetched from it
    pub fn resolve_space_position(
        query: &Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
        obj_id: ObjId,
    ) -> Option<LocationSpace> {
        match query.get(obj_id).ok()? {
            (_, Some(space), _) => Some(space.clone()),
            (_, _, Some(docked)) => Self::resolve_space_position(query, docked.parent_id),
            _ => None,
        }
    }

    pub fn is_docked_at(
        query_locations: &Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
        obj_id: ObjId,
        target_id: ObjId,
    ) -> bool {
        query_locations
            .get_component::<LocationDocked>(obj_id)
            .map(|value| value.parent_id == target_id)
            .unwrap_or(false)
    }
}

pub fn force_update_locations_index(world: &mut World) {
    world.run_system_once(update_entity_per_sector_index);
}

pub fn update_entity_per_sector_index(
    mut index: ResMut<EntityPerSectorIndex>,
    query: Query<(
        Entity,
        &LocationSpace,
        Option<&Extractable>,
        Option<&HasDocking>,
    )>,
) {
    log::trace!("running");
    index.clear();

    for (obj_id, location, maybe_extratable, maybe_docking) in &query {
        let sector_id = location.sector_id;

        // log::trace!("indexing {:?} at {:?}", entity, sector_id);
        index.add(sector_id, obj_id);

        if maybe_extratable.is_some() {
            // log::trace!("indexing extractable {:?} at {:?}", entity, sector_id);
            index.add_extractable(sector_id, obj_id);
        }

        if maybe_docking.is_some() {
            // log::trace!("indexing stations {:?} at {:?}", entity, sector_id);
            index.add_stations(sector_id, obj_id);
        }
    }
}
