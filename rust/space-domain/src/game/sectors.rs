use std::collections::HashMap;

use shred::World;
use specs::prelude::*;
use specs_derive::*;

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
pub struct Sector {}

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
        ReadStorage<'a, Sector>,
        ReadStorage<'a, Jump>,
        ReadStorage<'a, Location>,
    );

    fn run(&mut self, (entities, sectors, jumps, locations): Self::SystemData) {}
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

        let sector_0 = world.create_entity().with(Sector {}).build();
        let sector_1 = world.create_entity().with(Sector {}).build();
        let sector_2 = world.create_entity().with(Sector {}).build();
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
            let mut list = vec![];

            // TODO: optimize search
            for (e, j, l) in (entities, jumps, locations).join() {
                if l.get_sector_id() == Some(*current) {
                    list.push(j.target_sector_id);
                }
            }

            list
        },
        |current| *current == to,
    )?;

    let mut result = vec![];
    for (from, to) in path.into_iter().tuple_windows() {
        // TODO: optimize search
        for (e, j, l) in (entities, jumps, locations).join() {
            if l.get_sector_id() == Some(from) && j.target_sector_id == to {
                result.push(PathLeg {
                    sector_id: from,
                    jump_id: e,
                    jump_pos: l.get_pos().unwrap(),
                    target_sector_id: to,
                    target_pos: j.target_pos,
                });
            }
        }
    }

    Some(result)

    // path.into_iter()
    //     .tuple_windows()
    //     .map(|(a, b)| PathLeg {
    //         sector_id: (),
    //         jump_id: (),
    //         jump_pos: V2 {},
    //         target_jump_id: (),
    //         target_sector_id: (),
    //         target_pos: V2 {},
    //     })
    //     .collect()
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
