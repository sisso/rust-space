use std::collections::HashMap;

use shred::{Read, ResourceId, SystemData, World, Write};
use specs::prelude::*;
use specs_derive::*;

use crate::utils::*;
use crate::game::objects::ObjId;
use specs::world::EntitiesRes;
use crate::game::locations::Location;
use crate::game::{RequireInitializer, GameInitContext};

#[derive(Clone, Debug, Component)]
pub struct Jump {
    pub target_id: ObjId,
}

pub type JumpId = Entity;
pub type SectorId = Entity;

#[derive(Clone, Debug, Component)]
pub struct Sector {
}

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
    jumps: Vec<IndexedJump>,
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
    pub fn update_index_from_world(world: &mut World) {
        let entities = &world.entities();
        let sectors_storage = &world.read_storage::<Sector>();
        let jumps_storage = &world.read_storage::<Jump>();
        let locations_storage = &world.read_storage::<Location>();
        let sectors_index = &mut world.write_resource::<SectorsIndex>();
        sectors_index.update_index(entities, sectors_storage, jumps_storage, locations_storage);
    }

    pub fn update_index(&mut self, entities: &Read<EntitiesRes>,
                        sectors_storage: &ReadStorage<Sector>,
                        jump_storage: &ReadStorage<Jump>,
                        locations_storage: &ReadStorage<Location>) {
        self.jumps.clear();

        let mut jumps_data: HashMap<Entity, (SectorId, Position, JumpId)> = Default::default();

        for (entity, jump, location) in (entities, jump_storage, locations_storage).join() {
            let (pos, sector_id) = match location {
                Location::Space { pos, sector_id} => (pos, sector_id),
                _ => panic!("{:?} jump has invalid location {:?}", entity, location),
            };

            jumps_data.insert(entity, (
                *sector_id,
                *pos,
                jump.target_id,
            ));
        }

        for (jump_id, (sector_id, pos, target_jump_id)) in jumps_data.iter() {
            let (target_sector, target_pos, _) = jumps_data.get(target_jump_id).unwrap();

            let entry = IndexedJump {
                id: *jump_id,
                from_sector_id: *sector_id,
                to_sector_id: *target_sector,
                from_pos: *pos,
                to_pos: *target_pos,
            };

            self.jumps.push(entry);
        }
    }

    pub fn find_jump(&self, from_sector_id: SectorId, to_sector_id: SectorId) -> Option<IndexedJump>{
        self.jumps.iter().find(|jump| {
            jump.from_sector_id == from_sector_id &&
                jump.to_sector_id == to_sector_id
        }).cloned()
    }
}

pub mod test_scenery {
    use super::*;
    use crate::game::Game;
    use crate::game::locations::Location;

    #[derive(Debug)]
    pub struct SectorScenery {
        pub sector_0: ObjId,
        pub sector_1: ObjId,
        pub jump_0_to_1: ObjId,
        pub jump_0_to_1_pos: Position,
        pub jump_1_to_0: ObjId,
        pub jump_1_to_0_pos: Position,
    }

    /// Setup 2 sector with jump gate connecting
    pub fn setup_sector_scenery(world: &mut World) -> SectorScenery {
        world.register::<Location>();
        world.register::<Jump>();
        world.register::<Sector>();
        world.insert(SectorsIndex::default());

        let sector_0 = world.create_entity().build();
        let sector_1 = world.create_entity().build();
        let jump_0_to_1 = world.create_entity().build();
        let jump_1_to_0 = world.create_entity().build();
        let jump_0_to_1_pos = V2::new(4.0, 1.0);
        let jump_1_to_0_pos = V2::new(-2.0, -3.0);

        // encapsulate storage because the destructors hold mutability in world
        {
            let sectors = &mut world.write_storage::<Sector>();
            sectors.insert(sector_0, Sector {
                // jumps: vec![
                //     jump_0_to_1
                // ]
            }).unwrap();

            sectors.insert(sector_1, Sector {
                // jumps: vec![
                //     jump_1_to_0
                // ]
            }).unwrap();

            let jumps = &mut world.write_storage::<Jump>();
            jumps.insert(jump_0_to_1, Jump {
                target_id: jump_1_to_0,
            }).unwrap();

            jumps.insert(jump_1_to_0, Jump {
                target_id: jump_0_to_1,
            }).unwrap();

            let locations = &mut world.write_storage::<Location>();
            locations.insert(jump_0_to_1, Location::Space {
                pos: jump_0_to_1_pos,
                sector_id: sector_0,
            }).unwrap();

            locations.insert(jump_1_to_0, Location::Space {
                pos: jump_1_to_0_pos,
                sector_id: sector_1,
            }).unwrap();
        }

        SectorsIndex::update_index_from_world(world);

        SectorScenery {
            sector_0,
            sector_1,
            jump_0_to_1,
            jump_1_to_0,
            jump_0_to_1_pos,
            jump_1_to_0_pos,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::sectors::{SectorsIndex, Sector, Jump};
    use specs::{World, WorldExt, Builder};
    use crate::game::locations::Location;

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
