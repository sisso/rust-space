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
use crate::game::navigations::{Navigation, NavigationMoveTo, Navigations};

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

///
/// Undock all docked miners that have a target
///
pub struct UndockMinersWithTargetSystem;

#[derive(SystemData)]
pub struct UndockMinersWithTargetData<'a> {
    entities: Entities<'a>,
    locations_dock: ReadStorage<'a, LocationDock>,
    commands_mine_target: ReadStorage<'a, CommandMineTarget>,
    actions_undock: WriteStorage<'a, ActionUndock>,
}

impl<'a> System<'a> for UndockMinersWithTargetSystem {
    type SystemData = UndockMinersWithTargetData<'a>;

    fn run(&mut self, mut data: UndockMinersWithTargetData) {
        use specs::Join;

        let mut to_add = vec![];

        for (e, _, _, _) in (&*data.entities, &data.locations_dock, &data.commands_mine_target, !&data.actions_undock).join() {
            to_add.push(e);
        }

        for e in to_add {
            data.actions_undock.insert(e, ActionUndock);
        }
    }
}

pub struct SetupNavigationForMinersSystem;

///
/// Setup navigation for all undocked miners with target
///
#[derive(SystemData)]
pub struct SetupNavigationForMinersData<'a> {
    entities: Entities<'a>,
    sectors: Read<'a, Sectors>,
    locations_sector_id: ReadStorage<'a, LocationSector>,
    locations_positions: ReadStorage<'a, LocationSpace>,
    commands_mine_target: ReadStorage<'a, CommandMineTarget>,
    navigations: WriteStorage<'a, Navigation>,
    navigations_move_to: WriteStorage<'a, NavigationMoveTo>,
}

impl<'a> System<'a> for SetupNavigationForMinersSystem {
    type SystemData = SetupNavigationForMinersData<'a>;

    fn run(&mut self, mut data: SetupNavigationForMinersData) {
        use specs::Join;

        let mut to_add = BitSet::new();
        let mut targets = BitSet::new();

        for (e, target, _) in (&data.entities, &data.commands_mine_target, !&data.navigations).join() {
            to_add.add(e.id());
            targets.add(target.target_obj_id.id());
        }

        let mut targets_by_pos = HashMap::new();
        for (id, entity, sector, pos) in (&targets, &data.entities, &data.locations_sector_id, &data.locations_positions).join() {
            targets_by_pos.insert(id, (sector.sector_id, pos.pos));
        }

        let sectors = data.sectors.borrow();

        for (_, entity, target, sector, position) in (&to_add, &data.entities, &data.commands_mine_target, &data.locations_sector_id, &data.locations_positions).join() {
            let (target_sector_id, target_pos) = targets_by_pos.get(&target.target_obj_id.id()).unwrap();

            let plan = Navigations::create_plan(
                sectors,
                sector.sector_id,
                position.pos.clone(),
                *target_sector_id,
                target_pos.clone()
            );

            data.navigations.insert(entity, Navigation::MoveTo);
            data.navigations_move_to.insert(entity, NavigationMoveTo {
                target: target.target_obj_id,
                plan: plan,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use specs::DispatcherBuilder;
    use crate::game::wares::WareId;
    use crate::game::locations::LocationSector;

    struct SceneryRequest {
    }

    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
    }

    const SECTOR_0: SectorId = SectorId(0);
    const SECTOR_1: SectorId = SectorId(1);
    const JUMP_0_TO_1: Jump = Jump {
        id: JumpId(0),
        sector_id: SECTOR_0,
        pos: Position { x: 5.0, y: 0.0 },
        to_sector_id: SECTOR_1,
        to_pos: Position { x: 0.0, y: 5.0 },
    };
    const JUMP_1_TO_0: Jump = Jump {
        id: JumpId(1),
        sector_id: SECTOR_0,
        pos: Position { x: 5.0, y: 0.0 },
        to_sector_id: SECTOR_1,
        to_pos: Position { x: 0.0, y: 5.0 },
    };
    const WARE_0: WareId = WareId(0);
    const EXTRACTABLE: Extractable = Extractable { ware_id: WARE_0, time: DeltaTime(1.0) };

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let asteroid =
            world.create_entity()
                .with(LocationSector { sector_id: SECTOR_1 })
                .with(EXTRACTABLE)
                .build();

        let miner =
            world.create_entity()
                .with(LocationSector { sector_id: SECTOR_0 })
                .with(CommandMine {})
                .build();

        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        world.insert(entitys_per_sector);
        
        SceneryResult {
            miner,
            asteroid,
        }
    }

    #[test]
    fn test_command_mine_search_targets() {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(SearchMineTargetsSystem, "miner_search_targets", &[])
            .build();
        dispatcher.setup(&mut world);

        let scenery = setup_scenery(&mut world);

        dispatcher.dispatch(&world);
        world.maintain();

        let command_storage = world.read_component::<CommandMineTarget>();
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

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(SearchMineTargetsSystem, "miner_search_targets", &[])
            .build();
        dispatcher.setup(&mut world);

        let scenery = setup_scenery(&mut world);

        dispatcher.dispatch(&world);

        let command_storage = world.read_component::<CommandMineTarget>();
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

    #[test]
    fn test_command_mine_should_undock_entities_with_target() {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(UndockMinersWithTargetSystem, "", &[])
            .build();
        dispatcher.setup(&mut world);

        let asteroid = world.create_entity()
            .build();

        let station = world.create_entity()
            .build();

        let miner = world.create_entity()
            .with(LocationDock { docked_id: station })
            .with(CommandMineTarget { target_obj_id: asteroid })
            .build();

        dispatcher.dispatch(&world);

        let storage = world.read_component::<ActionUndock>();
        let action = storage.get(miner);
        assert!(action.is_some());
    }

    fn setup_sectors() -> Sectors {
        let mut sectors = Sectors::new();

        sectors.add_sector(Sector { id: SECTOR_0 });
        sectors.add_sector(Sector { id: SECTOR_1 });
        sectors.add_jump(JUMP_0_TO_1.clone());
        sectors.add_jump(JUMP_1_TO_0.clone());

        sectors
    }

    #[test]
    fn test_command_mine_with_target_should_set_navigation() {
        let mut world = World::new();
        world.insert(setup_sectors());

        let mut dispatcher = DispatcherBuilder::new()
            .with(SetupNavigationForMinersSystem, "", &[])
            .build();
        dispatcher.setup(&mut world);

        let asteroid = world.create_entity()
            .with(LocationSpace { pos: Position::new(10.0, 0.0) })
            .with(LocationSector{ sector_id: SECTOR_1 })
            .build();

        let miner = world.create_entity()
            .with(LocationSpace { pos: Position::new(0.0, 0.0) })
            .with(LocationSector{ sector_id: SECTOR_0 })
            .with(CommandMineTarget { target_obj_id: asteroid })
            .build();

        dispatcher.dispatch(&world);

        let storage = world.read_component::<NavigationMoveTo>();
        let nav = storage.get(miner);
        assert!(nav.is_some());
        let nav = nav.unwrap();
        assert_eq!(nav.target, asteroid);
        let plan = &nav.plan;
        assert_eq!(plan.target_sector_id, SECTOR_1);
        assert_eq!(plan.target_position, Position::new(10.0, 0.0));
        assert_eq!(plan.path.len(), 5);
    }
}
