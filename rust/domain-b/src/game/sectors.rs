use bevy_ecs::prelude::*;
use std::borrow::BorrowMut;

use bevy_ecs::system::SystemState;
use commons::math::{IntoP2Ext, P2, P2I};
use std::time::Instant;

use crate::game::locations::LocationSpace;
use crate::game::objects::ObjId;
use crate::game::utils::*;

#[derive(Clone, Debug, Component)]
pub struct Jump {
    pub target_sector_id: SectorId,
    pub target_pos: P2,
}

pub type JumpId = Entity;
pub type SectorId = Entity;

#[derive(Clone, Debug, Component)]
pub struct JumpCache {
    pub jump_id: Entity,
    pub to_sector: Entity,
}

#[derive(Clone, Debug, Component)]
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

pub struct Sectors;

pub fn update_sectors_index(world: &World) {
    todo!()
    // let mut dispatcher = DispatcherBuilder::new()
    //     .with(UpdateIndexSystem, "index", &[])
    //     .build();
    // dispatcher.run_now(world);
}

// pub struct UpdateIndexSystem;
//
// impl<'a> System<'a> for UpdateIndexSystem {
//     type SystemData = (
//         Entities<'a>,
//         WriteStorage<'a, Sector>,
//         ReadStorage<'a, Jump>,
//         ReadStorage<'a, LocationSpace>,
//     );
//
//     fn run(&mut self, (entities, mut sectors, jumps, locations): Self::SystemData) {
//         log::debug!("indexing sectors");
//         let start = Instant::now();
//         let sectors = sectors.borrow_mut();
//
//         for (e, j, l) in (&entities, &jumps, &locations).join() {
//             let cache = JumpCache {
//                 jump_id: e,
//                 to_sector: j.target_sector_id,
//             };
//
//             let sector = sectors.get_mut(l.sector_id).expect("jump sector not found");
//             match sector.jumps_cache.as_mut() {
//                 Some(list) => list.push(cache),
//                 None => sector.jumps_cache = Some(vec![cache]),
//             }
//         }
//
//         let total = Instant::now() - start;
//         log::debug!("indexing sector complete in {:?}", total);
//     }
// }
//
// pub mod test_scenery {
//     use super::*;
//     use crate::game::label::Label;
//
//     #[derive(Debug)]
//     pub struct SectorScenery {
//         pub sector_0: ObjId,
//         pub sector_1: ObjId,
//         pub sector_2: ObjId,
//         pub jump_0_to_1: ObjId,
//         pub jump_0_to_1_pos: P2,
//         pub jump_1_to_0: ObjId,
//         pub jump_1_to_0_pos: P2,
//         pub jump_1_to_2: ObjId,
//         pub jump_1_to_2_pos: P2,
//         pub jump_2_to_1: ObjId,
//         pub jump_2_to_1_pos: P2,
//     }
//
//     /// Setup 3 sector with jump gate connecting
//     pub fn setup_sector_scenery(world: &mut World) -> SectorScenery {
//         world.register::<LocationSpace>();
//         world.register::<Jump>();
//         world.register::<Sector>();
//         world.register::<Label>();
//
//         let sector_0 = world
//             .create_entity()
//             .with(Sector::new(P2I::new(0, 0)))
//             .build();
//         let sector_1 = world
//             .create_entity()
//             .with(Sector::new(P2I::new(1, 0)))
//             .build();
//         let sector_2 = world
//             .create_entity()
//             .with(Sector::new(P2I::new(2, 0)))
//             .build();
//         let jump_0_to_1_pos = V2::new(0.0, 1.0);
//         let jump_1_to_0_pos = V2::new(1.0, 0.0);
//         let jump_1_to_2_pos = V2::new(1.0, 2.0);
//         let jump_2_to_1_pos = V2::new(2.0, 1.0);
//
//         let jump_0_to_1 = world
//             .create_entity()
//             .with(Jump {
//                 target_sector_id: sector_1,
//                 target_pos: jump_1_to_0_pos,
//             })
//             .with(LocationSpace {
//                 pos: jump_0_to_1_pos,
//                 sector_id: sector_0,
//             })
//             .build();
//
//         let jump_1_to_0 = world
//             .create_entity()
//             .with(Jump {
//                 target_sector_id: sector_0,
//                 target_pos: jump_0_to_1_pos,
//             })
//             .with(LocationSpace {
//                 pos: jump_1_to_0_pos,
//                 sector_id: sector_1,
//             })
//             .build();
//
//         let jump_1_to_2 = world
//             .create_entity()
//             .with(Jump {
//                 target_sector_id: sector_2,
//                 target_pos: jump_2_to_1_pos,
//             })
//             .with(LocationSpace {
//                 pos: jump_1_to_2_pos,
//                 sector_id: sector_1,
//             })
//             .build();
//
//         let jump_2_to_1 = world
//             .create_entity()
//             .with(Jump {
//                 target_sector_id: sector_2,
//                 target_pos: jump_1_to_2_pos,
//             })
//             .with(LocationSpace {
//                 pos: jump_2_to_1_pos,
//                 sector_id: sector_2,
//             })
//             .build();
//
//         update_sectors_index(world);
//
//         SectorScenery {
//             sector_0,
//             sector_1,
//             sector_2,
//             jump_0_to_1,
//             jump_0_to_1_pos,
//             jump_1_to_0,
//             jump_1_to_0_pos,
//             jump_1_to_2,
//             jump_1_to_2_pos,
//             jump_2_to_1,
//             jump_2_to_1_pos,
//         }
//     }
// }
//
// #[derive(Debug)]
// pub struct PathLeg {
//     pub sector_id: SectorId,
//     pub jump_id: JumpId,
//     pub jump_pos: P2,
//     pub target_sector_id: SectorId,
//     pub target_pos: P2,
// }
//
// pub fn find_path<'a>(
//     entities: &Entities<'a>,
//     sectors: &ReadStorage<'a, Sector>,
//     jumps: &ReadStorage<'a, Jump>,
//     locations: &ReadStorage<'a, LocationSpace>,
//     from: SectorId,
//     to: SectorId,
// ) -> Option<Vec<PathLeg>> {
//     find_path_raw(entities, sectors, jumps, locations, from, to, 0)
// }
//
// pub fn find_path_raw<'a>(
//     _entities: &Entities<'a>,
//     sectors: &ReadStorage<'a, Sector>,
//     jumps: &ReadStorage<'a, Jump>,
//     locations: &ReadStorage<'a, LocationSpace>,
//     from: SectorId,
//     to: SectorId,
//     algorithm: u8,
// ) -> Option<Vec<PathLeg>> {
//     use itertools::Itertools;
//     let start = Instant::now();
//
//     if from == to {
//         return Some(vec![]);
//     }
//
//     let mut count = 0;
//     let to_coords = sectors.get(to).unwrap().coords.as_p2();
//
//     let path: Vec<SectorId> = {
//         if algorithm == 0 {
//             pathfinding::prelude::astar(
//                 &from,
//                 |current| {
//                     let sector = sectors.get(*current).unwrap();
//                     let jump_cache = sector.jumps_cache.as_ref();
//                     let successors: Vec<(SectorId, u32)> = jump_cache
//                         .expect("sector jump cache is empty")
//                         .iter()
//                         .map(|i| (i.to_sector, 1))
//                         .collect();
//                     count += 1;
//                     successors
//                 },
//                 |current| {
//                     let current_coords = sectors.get(*current).unwrap().coords.as_p2();
//                     (to_coords - current_coords).length_squared() as u32 / 1000u32
//                 },
//                 |current| *current == to,
//             )
//             .map(|(path, _cost)| path)?
//         } else if algorithm == 1 {
//             pathfinding::prelude::astar(
//                 &from,
//                 |current| {
//                     let sector = sectors.get(*current).unwrap();
//                     let jump_cache = sector.jumps_cache.as_ref();
//                     let successors: Vec<(SectorId, u32)> = jump_cache
//                         .expect("sector jump cache is empty")
//                         .iter()
//                         .map(|i| (i.to_sector, 1))
//                         .collect();
//                     count += 1;
//                     successors
//                 },
//                 |current| {
//                     let current_coords = sectors.get(*current).unwrap().coords;
//                     (pathfinding::prelude::absdiff(current_coords.x as f32, to_coords.x)
//                         + pathfinding::prelude::absdiff(current_coords.y as f32, to_coords.y))
//                         as u32
//                 },
//                 |current| *current == to,
//             )
//             .map(|(path, _cost)| path)?
//         } else if algorithm == 3 {
//             pathfinding::prelude::astar(
//                 &from,
//                 |current| {
//                     let sector = sectors.get(*current).unwrap();
//                     let jump_cache = sector.jumps_cache.as_ref();
//                     let successors: Vec<(SectorId, u32)> = jump_cache
//                         .expect("sector jump cache is empty")
//                         .iter()
//                         .map(|i| (i.to_sector, 1))
//                         .collect();
//                     count += 1;
//                     successors
//                 },
//                 |current| {
//                     let current_coords = sectors.get(*current).unwrap().coords;
//                     (pathfinding::prelude::absdiff(current_coords.x as f32, to_coords.x)
//                         + pathfinding::prelude::absdiff(current_coords.y as f32, to_coords.y))
//                         as u32
//                 },
//                 |current| *current == to,
//             )
//             .map(|(path, _cost)| path)?
//         } else {
//             pathfinding::prelude::bfs(
//                 &from,
//                 |current| {
//                     let sector = sectors.get(*current).unwrap();
//                     let jump_cache = sector.jumps_cache.as_ref();
//                     let successors: Vec<SectorId> = jump_cache
//                         .expect("sector jump cache is empty")
//                         .iter()
//                         .map(|i| i.to_sector)
//                         .collect();
//                     count += 1;
//                     successors
//                 },
//                 |current| *current == to,
//             )?
//         }
//     };
//
//     let mut result = vec![];
//     for (from, to) in path.into_iter().tuple_windows() {
//         let sector = sectors.get(from).unwrap();
//
//         let jc = sector
//             .jumps_cache
//             .as_ref()
//             .and_then(|i| i.iter().find(|j| j.to_sector == to))
//             .unwrap();
//
//         let jump_pos = locations
//             .get(jc.jump_id)
//             .expect("jump id has no location")
//             .pos;
//         let jump_target_pos = &jumps
//             .get(jc.jump_id)
//             .expect("jump id has no jump")
//             .target_pos;
//
//         result.push(PathLeg {
//             sector_id: from,
//             jump_id: jc.jump_id,
//             jump_pos: jump_pos,
//             target_sector_id: to,
//             target_pos: *jump_target_pos,
//         });
//     }
//
//     let plan_complete = Instant::now();
//     let duration = plan_complete - start;
//     if duration > std::time::Duration::from_millis(1) {
//         let from_coords = sectors.get(from).unwrap().coords;
//
//         log::warn!(
//             "create plan find_path {:?}, number of edges {}, number of query nodes {}, from {:?} to {:?}",
//             duration,
//             result.len(),
//             count,
//             from_coords,
//             to_coords
//         );
//     }
//
//     Some(result)
// }

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

// #[cfg(test)]
// mod test {
//     use super::test_scenery::setup_sector_scenery;
//     use crate::game::events::Events;
//
//     use crate::game::sectors::{Jump, PathLeg, Sector, SectorId};
//
//     use bevy_ecs::prelude::*;
//
//     use crate::game::label::Label;
//     use crate::game::locations::LocationSpace;
//     use commons::math::P2I;
//     use std::time::Instant;
//
//     #[test]
//     fn test_find_path_same_sector() {
//         let mut world = World::new();
//         let sector_scenery = setup_sector_scenery(&mut world);
//         let path =
//             do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_0).unwrap();
//         assert_eq!(0, path.len());
//     }
//
//     #[test]
//     fn test_find_path_one() {
//         let mut world = World::new();
//         let sector_scenery = setup_sector_scenery(&mut world);
//         let path =
//             do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_1).unwrap();
//         assert_eq!(1, path.len());
//         assert_eq!(sector_scenery.jump_0_to_1, path[0].jump_id);
//         assert_eq!(sector_scenery.jump_0_to_1_pos, path[0].jump_pos);
//     }
//
//     #[test]
//     fn test_find_path_two() {
//         let mut world = World::new();
//         let sector_scenery = setup_sector_scenery(&mut world);
//         let path =
//             do_find_path(&mut world, sector_scenery.sector_0, sector_scenery.sector_2).unwrap();
//         assert_eq!(2, path.len());
//         assert_eq!(sector_scenery.jump_0_to_1, path[0].jump_id);
//         assert_eq!(sector_scenery.jump_0_to_1_pos, path[0].jump_pos);
//         assert_eq!(sector_scenery.jump_1_to_2, path[1].jump_id);
//         assert_eq!(sector_scenery.jump_1_to_2_pos, path[1].jump_pos);
//     }
//
//     #[test]
//     fn test_find_path_performance() {
//         _ = env_logger::builder()
//             .filter(None, log::LevelFilter::Warn)
//             .try_init();
//
//         let mut world = World::new();
//         world.register::<Sector>();
//         world.register::<LocationSpace>();
//         world.register::<Jump>();
//         world.register::<Label>();
//         world.insert(Events::default());
//
//         // [2021-11-13T12:45:56Z WARN  space_domain::game::sectors] create plan find_path 2.761989ms
//         // number of edges 87, number of query nodes 1864,
//         // from V2 { x: 12.0, y: 7.0 } to V2 { x: 31.0, y: 45.0 }
//         let size = (100, 100);
//         let p1 = P2I::new(12, 7);
//         let p2 = P2I::new(31, 45);
//
//         crate::game::scenery_random::generate_sectors(&mut world, size, 13801247937784236795);
//
//         let entities = &world.entities();
//         let sectors = &world.read_storage::<Sector>();
//         let jumps = &world.read_storage::<Jump>();
//         let locations = &world.read_storage::<LocationSpace>();
//
//         let from = super::get_sector_by_coords(entities, sectors, p1).unwrap();
//         let to = super::get_sector_by_coords(entities, sectors, p2).unwrap();
//
//         let mut times = vec![];
//         for alg in vec![0, 1, 2, 3] {
//             let start = Instant::now();
//             super::find_path_raw(entities, sectors, jumps, locations, from, to, alg);
//             let end = Instant::now();
//             times.push(end - start);
//         }
//
//         // should run on release mode
//         // assert!(
//         //     times
//         //         .iter()
//         //         .filter(|i| **i >= Duration::from_millis(1))
//         //         .count()
//         //         == 0,
//         //     "pathfind took {:?}",
//         //     times
//         // );
//     }
//
//     fn do_find_path(world: &mut World, from: SectorId, to: SectorId) -> Option<Vec<PathLeg>> {
//         super::find_path(
//             &world.entities(),
//             &world.read_storage::<Sector>(),
//             &world.read_storage::<Jump>(),
//             &world.read_storage::<LocationSpace>(),
//             from,
//             to,
//         )
//     }
// }
