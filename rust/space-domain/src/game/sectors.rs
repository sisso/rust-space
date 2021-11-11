use shred::World;
use specs::prelude::*;
use specs_derive::*;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::Instant;

use crate::game::locations::Location;
use crate::game::objects::ObjId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::*;

#[derive(Clone, Debug, Component)]
pub struct Jump {
    pub target_sector_id: SectorId,
    pub target_pos: Position,
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
    pub coords: Position,
    pub jumps_cache: Option<Vec<JumpCache>>,
}

impl Sector {
    pub fn new(coords: Position) -> Self {
        Sector {
            coords,
            jumps_cache: None,
        }
    }
}

pub struct Sectors;

impl RequireInitializer for Sectors {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Jump>();
        context.world.register::<Sector>();
    }
}

pub fn update_sectors_index(world: &mut World) {
    let mut dispatcher = DispatcherBuilder::new()
        .with(UpdateIndexSystem, "index", &[])
        .build();
    dispatcher.run_now(world);
}

pub struct UpdateIndexSystem;

impl<'a> System<'a> for UpdateIndexSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Sector>,
        ReadStorage<'a, Jump>,
        ReadStorage<'a, Location>,
    );

    fn run(&mut self, (entities, mut sectors, jumps, locations): Self::SystemData) {
        info!("indexing sectors");
        let start = Instant::now();
        let sectors = sectors.borrow_mut();

        for (e, j, l) in (&entities, &jumps, &locations).join() {
            let cache = JumpCache {
                jump_id: e,
                to_sector: j.target_sector_id,
            };

            let sector_id = l.get_sector_id().expect("jump must be in a sector");
            let sector = sectors.get_mut(sector_id).expect("jump sector not found");

            match sector.jumps_cache.as_mut() {
                Some(list) => list.push(cache),
                None => sector.jumps_cache = Some(vec![cache]),
            }
        }

        let total = Instant::now() - start;
        info!("indexing sector complete in {:?}", total);
    }
}

pub mod test_scenery {
    use super::*;
    use crate::game::locations::Location;

    #[derive(Debug)]
    pub struct SectorScenery {
        pub sector_0: ObjId,
        pub sector_1: ObjId,
        pub sector_2: ObjId,
        pub jump_0_to_1: ObjId,
        pub jump_0_to_1_pos: Position,
        pub jump_1_to_0: ObjId,
        pub jump_1_to_0_pos: Position,
        pub jump_1_to_2: ObjId,
        pub jump_1_to_2_pos: Position,
        pub jump_2_to_1: ObjId,
        pub jump_2_to_1_pos: Position,
    }

    /// Setup 3 sector with jump gate connecting
    pub fn setup_sector_scenery(world: &mut World) -> SectorScenery {
        world.register::<Location>();
        world.register::<Jump>();
        world.register::<Sector>();

        let sector_0 = world
            .create_entity()
            .with(Sector::new(V2::new(0.0, 0.0)))
            .build();
        let sector_1 = world
            .create_entity()
            .with(Sector::new(V2::new(1.0, 0.0)))
            .build();
        let sector_2 = world
            .create_entity()
            .with(Sector::new(V2::new(2.0, 0.0)))
            .build();
        let jump_0_to_1_pos = V2::new(0.0, 1.0);
        let jump_1_to_0_pos = V2::new(1.0, 0.0);
        let jump_1_to_2_pos = V2::new(1.0, 2.0);
        let jump_2_to_1_pos = V2::new(2.0, 1.0);

        let jump_0_to_1 = world
            .create_entity()
            .with(Jump {
                target_sector_id: sector_1,
                target_pos: jump_1_to_0_pos,
            })
            .with(Location::Space {
                pos: jump_0_to_1_pos,
                sector_id: sector_0,
            })
            .build();

        let jump_1_to_0 = world
            .create_entity()
            .with(Jump {
                target_sector_id: sector_0,
                target_pos: jump_0_to_1_pos,
            })
            .with(Location::Space {
                pos: jump_1_to_0_pos,
                sector_id: sector_1,
            })
            .build();

        let jump_1_to_2 = world
            .create_entity()
            .with(Jump {
                target_sector_id: sector_2,
                target_pos: jump_2_to_1_pos,
            })
            .with(Location::Space {
                pos: jump_1_to_2_pos,
                sector_id: sector_1,
            })
            .build();

        let jump_2_to_1 = world
            .create_entity()
            .with(Jump {
                target_sector_id: sector_2,
                target_pos: jump_1_to_2_pos,
            })
            .with(Location::Space {
                pos: jump_2_to_1_pos,
                sector_id: sector_2,
            })
            .build();

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

#[derive(Debug)]
pub struct PathLeg {
    pub sector_id: SectorId,
    pub jump_id: JumpId,
    pub jump_pos: Position,
    pub target_sector_id: SectorId,
    pub target_pos: Position,
}

pub fn find_path<'a>(
    entities: &Entities<'a>,
    sectors: &ReadStorage<'a, Sector>,
    jumps: &ReadStorage<'a, Jump>,
    locations: &ReadStorage<'a, Location>,
    from: SectorId,
    to: SectorId,
) -> Option<Vec<PathLeg>> {
    use itertools::Itertools;
    use pathfinding::prelude::bfs;

    if from == to {
        return Some(vec![]);
    }

    let path = bfs(
        &from,
        |current| {
            let sector = sectors.get(*current).unwrap();
            let jump_cache = sector.jumps_cache.as_ref();
            let successors: Vec<SectorId> = jump_cache
                .expect("sector jump cache is empty")
                .iter()
                .map(|i| i.to_sector)
                .collect();
            successors
        },
        |current| *current == to,
    )?;

    let mut result = vec![];
    for (from, to) in path.into_iter().tuple_windows() {
        let sector = sectors.get(from).unwrap();

        let jc = sector
            .jumps_cache
            .as_ref()
            .and_then(|i| i.iter().find(|j| j.to_sector == to))
            .unwrap();

        let jump_pos = locations
            .get(jc.jump_id)
            .expect("jump id has no location")
            .get_pos()
            .unwrap();
        let jump_target_pos = &jumps
            .get(jc.jump_id)
            .expect("jump id has no jump")
            .target_pos;

        result.push(PathLeg {
            sector_id: from,
            jump_id: jc.jump_id,
            jump_pos: *jump_pos,
            target_sector_id: to,
            target_pos: *jump_target_pos,
        });
    }

    Some(result)
}

#[cfg(test)]
mod test {
    use super::test_scenery::setup_sector_scenery;
    use crate::game::locations::Location;
    use crate::game::objects::ObjId;
    use crate::game::sectors::{Jump, PathLeg, Sector, SectorId};
    use specs::prelude::*;

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

    fn do_find_path(world: &mut World, from: SectorId, to: SectorId) -> Option<Vec<PathLeg>> {
        super::find_path(
            &world.entities(),
            &world.read_storage::<Sector>(),
            &world.read_storage::<Jump>(),
            &world.read_storage::<Location>(),
            from,
            to,
        )
    }
}
