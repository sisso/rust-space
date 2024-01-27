use bevy_ecs::prelude::*;
use std::collections::HashMap;

use bevy_ecs::system::RunSystemOnce;
use commons::math::{IntoP2Ext, P2, P2I};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::game::locations::LocationSpace;
use crate::game::objects::ObjId;
use crate::game::save::LoadingMapEntity;
use crate::game::utils::*;

pub type JumpId = Entity;
pub type SectorId = Entity;

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Jump {
    pub target_sector_id: SectorId,
    pub target_pos: P2,
}

impl LoadingMapEntity for Jump {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.target_sector_id.map_entity(entity_map);
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct JumpCache {
    pub jump_id: Entity,
    pub to_sector: Entity,
}

impl LoadingMapEntity for JumpCache {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.jump_id.map_entity(entity_map);
        self.to_sector.map_entity(entity_map);
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Sector {
    pub coords: P2I,
    pub jumps_cache: Option<Vec<JumpCache>>,
}

impl Sector {
    pub fn new(coords: P2I) -> Self {
        Sector {
            coords,
            jumps_cache: None,
        }
    }
}

impl LoadingMapEntity for Sector {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        if let Some(jc) = &mut self.jumps_cache {
            for i in jc {
                i.map_entity(entity_map);
            }
        }
    }
}

pub struct Sectors;

pub fn system_update_sectors_index(
    jumps: Query<(Entity, &Jump, &LocationSpace), Changed<Jump>>,
    mut sectors: Query<&mut Sector>,
) {
    log::trace!("indexing sectors");
    let start = Instant::now();

    for (jump_id, jump, location) in &jumps {
        let mut e_sector = sectors
            .get_mut(location.sector_id)
            .expect("sector_id not found");

        let jc = &mut e_sector.jumps_cache;
        if jc.is_none() {
            *jc = Some(vec![]);
        }

        let cache_list = jc.as_mut().unwrap();
        cache_list.retain(|i| i.jump_id != jump_id);
        let cache = JumpCache {
            jump_id: jump_id,
            to_sector: jump.target_sector_id,
        };
        cache_list.push(cache);
    }

    let total = Instant::now() - start;
    log::trace!("indexing sector complete in {:?}", total);
}

#[derive(Debug)]
pub struct PathLeg {
    pub sector_id: SectorId,
    pub jump_id: JumpId,
    pub jump_pos: P2,
    pub target_sector_id: SectorId,
    pub target_pos: P2,
}

pub fn find_path_from_world(
    world: &mut World,
    from: SectorId,
    to: SectorId,
) -> Option<Vec<PathLeg>> {
    world.run_system_once_with(
        FindPathParams {
            from,
            to,
            algorithm: 0,
        },
        find_path,
    )
}

pub struct FindPathParams {
    pub from: SectorId,
    pub to: SectorId,
    pub algorithm: u8,
}

impl FindPathParams {
    pub fn new(from: SectorId, to: SectorId) -> Self {
        FindPathParams {
            from,
            to,
            algorithm: 0,
        }
    }
}

pub fn find_path(
    In(params): In<FindPathParams>,
    sectors: Query<&Sector>,
    jumps: Query<(&Jump, &LocationSpace)>,
) -> Option<Vec<PathLeg>> {
    find_path_raw(&sectors, &jumps, params)
}

pub fn find_path_raw(
    sectors: &Query<&Sector>,
    jumps: &Query<(&Jump, &LocationSpace)>,
    params: FindPathParams,
) -> Option<Vec<PathLeg>> {
    use itertools::Itertools;
    let start = Instant::now();

    if params.from == params.to {
        return Some(vec![]);
    }

    let mut count = 0;
    let to_coords = sectors.get(params.to).unwrap().coords.as_p2();

    let path: Vec<SectorId> = {
        if params.algorithm == 0 {
            pathfinding::prelude::astar(
                &params.from,
                |current| {
                    let sector = sectors.get(*current).unwrap();
                    let jump_cache = sector.jumps_cache.as_ref();
                    let successors: Vec<(SectorId, u32)> = jump_cache
                        .expect("sector jump cache is empty")
                        .iter()
                        .map(|i| (i.to_sector, 1))
                        .collect();
                    count += 1;
                    successors
                },
                |current| {
                    let current_coords = sectors.get(*current).unwrap().coords.as_p2();
                    (to_coords - current_coords).length_squared() as u32 / 1000u32
                },
                |current| *current == params.to,
            )
            .map(|(path, _cost)| path)?
        } else if params.algorithm == 1 {
            pathfinding::prelude::astar(
                &params.from,
                |current| {
                    let sector = sectors.get(*current).unwrap();
                    let jump_cache = sector.jumps_cache.as_ref();
                    let successors: Vec<(SectorId, u32)> = jump_cache
                        .expect("sector jump cache is empty")
                        .iter()
                        .map(|i| (i.to_sector, 1))
                        .collect();
                    count += 1;
                    successors
                },
                |current| {
                    let current_coords = sectors.get(*current).unwrap().coords;
                    (pathfinding::prelude::absdiff(current_coords.x as f32, to_coords.x)
                        + pathfinding::prelude::absdiff(current_coords.y as f32, to_coords.y))
                        as u32
                },
                |current| *current == params.to,
            )
            .map(|(path, _cost)| path)?
        } else if params.algorithm == 3 {
            pathfinding::prelude::astar(
                &params.from,
                |current| {
                    let sector = sectors.get(*current).unwrap();
                    let jump_cache = sector.jumps_cache.as_ref();
                    let successors: Vec<(SectorId, u32)> = jump_cache
                        .expect("sector jump cache is empty")
                        .iter()
                        .map(|i| (i.to_sector, 1))
                        .collect();
                    count += 1;
                    successors
                },
                |current| {
                    let current_coords = sectors.get(*current).unwrap().coords;
                    (pathfinding::prelude::absdiff(current_coords.x as f32, to_coords.x)
                        + pathfinding::prelude::absdiff(current_coords.y as f32, to_coords.y))
                        as u32
                },
                |current| *current == params.to,
            )
            .map(|(path, _cost)| path)?
        } else {
            pathfinding::prelude::bfs(
                &params.from,
                |current| {
                    let sector = sectors.get(*current).unwrap();
                    let jump_cache = sector.jumps_cache.as_ref();
                    let successors: Vec<SectorId> = jump_cache
                        .expect("sector jump cache is empty")
                        .iter()
                        .map(|i| i.to_sector)
                        .collect();
                    count += 1;
                    successors
                },
                |current| *current == params.to,
            )?
        }
    };

    let mut result = vec![];
    for (from, to) in path.into_iter().tuple_windows() {
        let sector = sectors.get(from).unwrap();

        let jc = sector
            .jumps_cache
            .as_ref()
            .and_then(|i| i.iter().find(|j| j.to_sector == to))
            .unwrap();

        let (jump, location) = jumps.get(jc.jump_id).expect("jump_id not found");
        let jump_pos = location.pos;
        let jump_target_pos = jump.target_pos;

        result.push(PathLeg {
            sector_id: from,
            jump_id: jc.jump_id,
            jump_pos: jump_pos,
            target_sector_id: to,
            target_pos: jump_target_pos,
        });
    }

    let plan_complete = Instant::now();
    let duration = plan_complete - start;
    if duration > std::time::Duration::from_millis(1) {
        let from_coords = sectors.get(params.from).unwrap().coords;

        log::warn!(
            "create plan find_path {:?}, number of edges {}, number of query nodes {}, from {:?} to {:?}",
            duration,
            result.len(),
            count,
            from_coords,
            to_coords
        );
    }

    Some(result)
}

pub fn get_sector_by_coords(input: In<P2I>, query: Query<(Entity, &Sector)>) -> Option<Entity> {
    query.iter().find_map(|(id, sector)| {
        if sector.coords.eq(&input.0) {
            Some(id)
        } else {
            None
        }
    })
}

pub fn list(query: Query<(Entity, With<Sector>)>) -> Vec<Entity> {
    query.iter().map(|i| i.0).collect()
}

#[cfg(test)]
mod test {
    use super::test_scenery::setup_sector_scenery;

    use crate::game::sectors::{system_update_sectors_index, FindPathParams, PathLeg, SectorId};

    use bevy_ecs::prelude::*;

    use crate::game::events::GEvents;

    use bevy_ecs::system::RunSystemOnce;
    use commons::math::P2I;
    use std::time::Instant;

    #[test]
    fn test_find_path_same_sector() {
        let mut world = World::new();
        let sector_scenery = setup_sector_scenery(&mut world);
        let path =
            do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_0).unwrap();
        assert_eq!(0, path.len());
    }

    #[test]
    fn test_find_path_one() {
        let mut world = World::new();
        let sector_scenery = setup_sector_scenery(&mut world);
        let path =
            do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_1).unwrap();
        assert_eq!(1, path.len());
        assert_eq!(sector_scenery.jump_0_to_1, path[0].jump_id);
        assert_eq!(sector_scenery.jump_0_to_1_pos, path[0].jump_pos);
    }

    #[test]
    fn test_find_path_two() {
        let mut world = World::new();
        let sector_scenery = setup_sector_scenery(&mut world);
        let path =
            do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_2).unwrap();
        assert_eq!(2, path.len());
        assert_eq!(sector_scenery.jump_0_to_1, path[0].jump_id);
        assert_eq!(sector_scenery.jump_0_to_1_pos, path[0].jump_pos);
        assert_eq!(sector_scenery.jump_1_to_2, path[1].jump_id);
        assert_eq!(sector_scenery.jump_1_to_2_pos, path[1].jump_pos);
    }

    #[test]
    fn test_find_path_performance() {
        let mut world = World::new();
        world.insert_resource(GEvents::default());

        // [2021-11-13T12:45:56Z WARN  space_domain::game::sectors] create plan find_path 2.761989ms
        // number of edges 87, number of query nodes 1864,
        // from V2 { x: 12.0, y: 7.0 } to V2 { x: 31.0, y: 45.0 }
        let size = (100, 100);
        let p1 = P2I::new(12, 7);
        let p2 = P2I::new(31, 45);

        crate::game::scenery_random::generate_sectors(&mut world, size, 13801247937784236795);
        world.run_system_once(system_update_sectors_index);

        let from = world
            .run_system_once_with(p1, super::get_sector_by_coords)
            .unwrap();
        let to = world
            .run_system_once_with(p2, super::get_sector_by_coords)
            .unwrap();

        let mut times = vec![];
        for alg in vec![0, 1, 2, 3] {
            let start = Instant::now();
            world.run_system_once_with(
                FindPathParams {
                    from: from,
                    to: to,
                    algorithm: alg,
                },
                super::find_path,
            );

            let end = Instant::now();
            let delta = end - start;
            times.push(delta);
            // println!("running {:?} on {:?}", alg, delta);
        }

        // should run on release mode
        // assert!(
        //     times
        //         .iter()
        //         .filter(|i| **i >= Duration::from_millis(1))
        //         .count()
        //         == 0,
        //     "pathfind took {:?}",
        //     times
        // );
    }

    fn do_find_path(world: &mut World, from: SectorId, to: SectorId) -> Option<Vec<PathLeg>> {
        world.run_system_once_with(FindPathParams::new(from, to), super::find_path)
    }
}

pub mod test_scenery {
    use super::*;

    #[derive(Debug)]
    pub struct SectorScenery {
        pub sector_0: ObjId,
        pub sector_1: ObjId,
        pub sector_2: ObjId,
        pub jump_0_to_1: ObjId,
        pub jump_0_to_1_pos: P2,
        pub jump_1_to_0: ObjId,
        pub jump_1_to_0_pos: P2,
        pub jump_1_to_2: ObjId,
        pub jump_1_to_2_pos: P2,
        pub jump_2_to_1: ObjId,
        pub jump_2_to_1_pos: P2,
    }

    /// Setup 3 sector with jump gate connecting
    pub fn setup_sector_scenery(world: &mut World) -> SectorScenery {
        let sector_0 = world.spawn_empty().insert(Sector::new(P2I::new(0, 0))).id();
        let sector_1 = world.spawn_empty().insert(Sector::new(P2I::new(1, 0))).id();
        let sector_2 = world.spawn_empty().insert(Sector::new(P2I::new(2, 0))).id();
        let jump_0_to_1_pos = V2::new(0.0, 1.0);
        let jump_1_to_0_pos = V2::new(1.0, 0.0);
        let jump_1_to_2_pos = V2::new(1.0, 2.0);
        let jump_2_to_1_pos = V2::new(2.0, 1.0);

        let jump_0_to_1 = world
            .spawn_empty()
            .insert(Jump {
                target_sector_id: sector_1,
                target_pos: jump_1_to_0_pos,
            })
            .insert(LocationSpace {
                pos: jump_0_to_1_pos,
                sector_id: sector_0,
            })
            .id();

        let jump_1_to_0 = world
            .spawn_empty()
            .insert(Jump {
                target_sector_id: sector_0,
                target_pos: jump_0_to_1_pos,
            })
            .insert(LocationSpace {
                pos: jump_1_to_0_pos,
                sector_id: sector_1,
            })
            .id();

        let jump_1_to_2 = world
            .spawn_empty()
            .insert(Jump {
                target_sector_id: sector_2,
                target_pos: jump_2_to_1_pos,
            })
            .insert(LocationSpace {
                pos: jump_1_to_2_pos,
                sector_id: sector_1,
            })
            .id();

        let jump_2_to_1 = world
            .spawn_empty()
            .insert(Jump {
                target_sector_id: sector_2,
                target_pos: jump_1_to_2_pos,
            })
            .insert(LocationSpace {
                pos: jump_2_to_1_pos,
                sector_id: sector_2,
            })
            .id();

        world.run_system_once(system_update_sectors_index);

        SectorScenery {
            sector_0,
            sector_1,
            sector_2,
            jump_0_to_1,
            jump_0_to_1_pos,
            jump_1_to_0,
            jump_1_to_0_pos,
            jump_1_to_2,
            jump_1_to_2_pos,
            jump_2_to_1,
            jump_2_to_1_pos,
        }
    }
}
