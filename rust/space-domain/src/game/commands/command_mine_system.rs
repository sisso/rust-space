///
/// System plans:
///
/// - search for target for non assigned miners
/// - create navigation plans for new miners
/// - start mine for miners that arrival at target
/// - trace back plan for miners that have full cargo
/// - deliver cargo
///
///

use specs::prelude::*;
use shred::{Read, ResourceId, SystemData, World, Write};
use specs_derive::*;

use super::*;
use crate::game::locations::{LocationDock, EntityPerSectorIndex, LocationSector};
use std::borrow::{Borrow, BorrowMut};
use crate::game::extractables::Extractable;
use crate::game::navigations::{Navigation, NavigationMoveTo};

pub struct SearchMineTargetsSystem;

#[derive(SystemData)]
pub struct SearchMineTargetsData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    locations_sector_id: ReadStorage<'a, LocationSector>,
    commands_mine: WriteStorage<'a, CommandMine>,
    commands_mine_target: WriteStorage<'a, CommandMineTarget>,
}

impl<'a> System<'a> for SearchMineTargetsSystem {
    type SystemData = SearchMineTargetsData<'a>;

    fn run(&mut self, mut data: SearchMineTargetsData) {
        use specs::Join;

        // search extractable
        let mut extractables = vec![];

        for (entity, _) in (&data.entities, &data.extractables).join() {
            extractables.push(entity);
        }

        let mut selected = vec![];

        for (entity, _, _, location_sector_id) in (&data.entities, &data.commands_mine, !&data.commands_mine_target, &data.locations_sector_id).join() {
            let sector_id = location_sector_id.sector_id;

            // search for nearest?
            let target: &ObjId = extractables.iter().next().unwrap();

            // set mine command
            let command = CommandMineTarget {
                target_obj_id: target.clone()
            };

            selected.push((entity, command));
        }

        for (entity, state) in selected {
            data.commands_mine_target.insert(entity, state).unwrap();
        }
    }
}

pub struct CommandMineSystem;

#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    command_mine: ReadStorage<'a, CommandMine>,
    locations_dock: ReadStorage<'a, LocationDock>,
    locations_space: ReadStorage<'a, LocationSpace>,
//    mine_states: WriteStorage<'a, MineState>,
    has_actions:  WriteStorage<'a, HasAction>,
}

impl<'a> System<'a> for CommandMineSystem {
    type SystemData = CommandMineData<'a>;

    fn run(&mut self, data: CommandMineData) {
        use specs::Join;

//        // generate plans
//        for (_, _, _) in (&mine_commands, !&mine_states, !&actions) {
//            // search nearest mine
//            if dockeds.contains(e)
//
//        }
//
//        // schedule next plan step
//        for (_, state, _) in (&mine_commands, &mine_states, !&actions).join() {
//
//        }
    }
}



//#[derive(SystemData)]
//pub struct UndockMinersData<'a> {
//    entities: Entities<'a>,
//    states: ReadStorage<'a, MineState>,
//    locations: ReadStorage<'a, LocationDock>,
//    has_actions: WriteStorage<'a, HasAction>,
//    undock_actions: WriteStorage<'a, ActionUndock>,
//}
//
//pub struct UndockMinersSystem;
//impl<'a> System<'a> for UndockMinersSystem {
//    type SystemData = UndockMinersData<'a>;
//
//    fn run(&mut self, mut data: UndockMinersData) {
//        use specs::Join;
//
//        let mut to_add = vec![];
//        for (entity, _, _, _) in (&data.entities, &data.states, !&data.has_actions, &data.locations).join() {
//            to_add.push(entity.clone());
//        }
//
//        for entity in to_add {
//            data.undock_actions.insert(entity, ActionUndock);
//        }
//    }
//}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::wares::WareId;
    use crate::game::locations::LocationSector;

    struct SceneryRequest {
    }

    struct SceneryResult {
        world: World,
        miner: ObjId,
        asteroid: ObjId,
    }

    const SECTOR_0: SectorId = SectorId(0);
    const SECTOR_1: SectorId = SectorId(1);

    fn setup_world(scenery: SceneryRequest) -> SceneryResult {
        let mut world = World::new();

        super::Commands::init_world(&mut world);
        super::Extractables::init_world(&mut world);
        super::Locations::init_world(&mut world);

        let asteroid =
            world.create_entity()
                .with(LocationSpace { pos: Position::new(0.0, 0.0) })
                .with(LocationSector { sector_id: SECTOR_1 })
                .with(Extractable { ware_id: WareId(0), time: DeltaTime(1.0) })
                .build();

        let miner =
            world.create_entity()
                .with(LocationSpace { pos: Position::new(1.0, 0.0) })
                .with(LocationSector { sector_id: SECTOR_0 })
                .with(CommandMine {})
                .build();

        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);

        world.insert(entitys_per_sector);

        SceneryResult {
            world,
            miner,
            asteroid
        }
    }

    #[test]
    fn test_command_mine_search_targets() {
        let scenery_request = SceneryRequest {
        };

        let mut scenery = setup_world(scenery_request);

        let mut system = SearchMineTargetsSystem;
        system.run_now(&mut scenery.world);

        let command_storage = scenery.world.read_component::<CommandMineTarget>();
        let command = command_storage.get(scenery.miner);
        match command {
            Some(command) => {
                assert_eq!(command.target_obj_id, scenery.asteroid);
            },
            None => {
                panic!("miner has no commandmine");
            }
        }
    }
}
