use std::collections::HashMap;

use shred::{Read, World};
use specs::prelude::*;
use specs_derive::*;

use crate::game::locations::Location;
use crate::game::objects::ObjId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::*;
use specs::world::EntitiesRes;

#[derive(Clone, Debug, Component)]
pub struct Jump {
    pub target_id: ObjId,
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
        context.world.insert(SectorsIndex::default());
    }
}

/// Indexing of all sectors and jump points
#[derive(Debug, Clone, Component, Default)]
pub struct SectorsIndex {
    jumps: HashMap<SectorId, Vec<IndexedJump>>,
}

#[derive(Debug, Clone)]
pub struct IndexedJump {
    pub id: JumpId,
    pub from_sector_id: SectorId,
    pub to_sector_id: SectorId,
    pub from_pos: Position,
    pub to_pos: Position,
}

impl SectorsIndex {
    // pub fn update_index_from_world(world: &mut World) {
    //     let entities = &world.entities();
    //     let sectors_storage = &world.read_storage::<Sector>();
    //     let jumps_storage = &world.read_storage::<Jump>();
    //     let locations_storage = &world.read_storage::<Location>();
    //     let sectors_index = &mut world.write_resource::<SectorsIndex>();
    //     sectors_index.update_index(entities, sectors_storage, jumps_storage, locations_storage);
    // }
    //
    // pub fn update_index(
    //     &mut self,
    //     entities: &Read<EntitiesRes>,
    //     _sectors_storage: &ReadStorage<Sector>,
    //     jump_storage: &ReadStorage<Jump>,
    //     locations_storage: &ReadStorage<Location>,
    // ) {
    //     self.jumps.clear();
    //
    //     let mut jumps_data: HashMap<SectorId, (SectorId, Position, JumpId)> = Default::default();
    //
    //     // collect all jumps and index so later we can refer from and to positions
    //     for (entity, jump, location) in (entities, jump_storage, locations_storage).join() {
    //         let (pos, sector_id) = match location {
    //             Location::Space { pos, sector_id } => (pos, sector_id),
    //             _ => panic!("{:?} jump has invalid location {:?}", entity, location),
    //         };
    //
    //         println!("indexing {:?} to {:?}", sector_id, jump.target_id);
    //         jumps_data.insert(entity, (*sector_id, *pos, jump.target_id));
    //     }
    //
    //     for (jump_id, (sector_id, pos, target_jump_id)) in jumps_data.iter() {
    //         let (target_sector, target_pos, _) = match jumps_data.get(target_jump_id) {
    //             Some(v) => v,
    //             None => panic!(
    //                 "fail to find indexed jump data for jump_id {:?}",
    //                 target_jump_id
    //             ),
    //         };
    //
    //         let entry = IndexedJump {
    //             id: *jump_id,
    //             from_sector_id: *sector_id,
    //             to_sector_id: *target_sector,
    //             from_pos: *pos,
    //             to_pos: *target_pos,
    //         };
    //
    //         self.jumps.push(entry);
    //     }
    // }

    pub fn find_jump(
        &self,
        from_sector_id: SectorId,
        to_sector_id: SectorId,
    ) -> Option<IndexedJump> {
        self.jumps
            .get(&from_sector_id)
            .iter()
            .find_map(|list| list.iter().find(|j| j.to_sector_id == to_sector_id))
            .cloned()
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
        Write<'a, SectorsIndex>,
        ReadStorage<'a, Jump>,
        ReadStorage<'a, Location>,
    );

    fn run(&mut self, (entities, mut sector_index, jumps, locations): Self::SystemData) {
        sector_index.jumps.clear();

        // per jump, the sector, the position, and target jump
        let mut jumps_data: HashMap<JumpId, (SectorId, Position, JumpId)> = Default::default();

        // collect all jumps and index so later we can refer from and to positions
        for (jump_id, jump, location) in (&entities, &jumps, &locations).join() {
            let (pos, sector_id) = match location {
                Location::Space { pos, sector_id } => (pos, sector_id),
                _ => panic!("{:?} jump has invalid location {:?}", jump_id, location),
            };

            println!(
                "indexing jump {:?} at {:?} to jump_id {:?}",
                jump_id, sector_id, jump.target_id
            );
            jumps_data.insert(jump_id, (*sector_id, *pos, jump.target_id));
        }

        for (jump_id, (sector_id, pos, target_jump_id)) in jumps_data.iter() {
            let (target_sector, target_pos, _) = match jumps_data.get(target_jump_id) {
                Some(v) => v,
                None => panic!("could not found target jump_id {:?}", target_jump_id),
            };

            let entry = IndexedJump {
                id: *jump_id,
                from_sector_id: *sector_id,
                to_sector_id: *target_sector,
                from_pos: *pos,
                to_pos: *target_pos,
            };

            let list = sector_index.jumps.entry(*sector_id).or_insert(vec![]);
            println!(
                "indexing jump_id {:?} from {:?} at {:?} to sector {:?} at {:?}",
                jump_id, sector_id, pos, target_sector, target_pos
            );
            list.push(entry);
        }
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
        world.insert(SectorsIndex::default());

        let sector_0 = world.create_entity().with(Sector {}).build();
        let sector_1 = world.create_entity().with(Sector {}).build();
        let sector_2 = world.create_entity().with(Sector {}).build();
        let jump_0_to_1_pos = V2::new(0.0, 1.0);
        let jump_1_to_0_pos = V2::new(1.0, 0.0);
        let jump_1_to_2_pos = V2::new(1.0, 2.0);
        let jump_2_to_1_pos = V2::new(2.0, 1.0);

        panic!("target id should not be a sector, but the jump into the other sector");

        // encapsulate storage because the destructors hold mutability in world
        let mut jumps = vec![];
        for (from, from_pos, to) in vec![
            (&sector_0, &jump_0_to_1_pos, &sector_1),
            (&sector_1, &jump_1_to_0_pos, &sector_0),
            (&sector_1, &jump_1_to_2_pos, &sector_2),
            (&sector_2, &jump_2_to_1_pos, &sector_1),
        ] {
            let e = world
                .create_entity()
                .with(Jump { target_id: *from })
                .with(Location::Space {
                    pos: *from_pos,
                    sector_id: *to,
                })
                .build();
            jumps.push(e);
        }

        super::update_sectors_index(world);

        SectorScenery {
            sector_0,
            sector_1,
            sector_2,
            jump_0_to_1: jumps[0],
            jump_0_to_1_pos,
            jump_1_to_0: jumps[1],
            jump_1_to_0_pos,
            jump_1_to_2: jumps[2],
            jump_1_to_2_pos,
            jump_2_to_1: jumps[3],
            jump_2_to_1_pos,
        }
    }
}

#[cfg(test)]
mod test {

    use crate::game::loader::Loader;
    use crate::game::sectors::SectorsIndex;
    use crate::utils::V2_ZERO;
    use specs::{World, WorldExt};

    #[test]
    fn test_sector_index() {
        let mut world = World::new();
        let scenery = super::test_scenery::setup_sector_scenery(&mut world);

        let index = &world.read_resource::<SectorsIndex>();
        let jump = index.find_jump(scenery.sector_0, scenery.sector_1).unwrap();

        assert_eq!(jump.from_sector_id, scenery.sector_0);
        assert_eq!(jump.to_sector_id, scenery.sector_1);
        assert_eq!(jump.from_pos, scenery.jump_0_to_1_pos);
        assert_eq!(jump.to_pos, scenery.jump_1_to_0_pos);
    }
}
