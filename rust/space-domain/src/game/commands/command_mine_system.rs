use shred::{Read, ResourceId, SystemData, World, Write};
///
/// System plans:
///
/// - for miner command without target, find a target
/// - for miner command with target without nav, if near target, mine
/// - for miner command with target without nave, if far away, create navigation to target
/// - for miner command with target without nav and mine action, if full, move back
///
/// - search for target for non assigned miners
/// - create navigation plans for new miners
/// - start mine for miners that arrival at target
/// - trace back plan for miners that have full cargo
/// - deliver cargo
///
///
use specs::prelude::*;
use specs_derive::*;

use super::*;
use crate::game::extractables::Extractable;
use crate::game::locations::{EntityPerSectorIndex, LocationDock, LocationSector};
use crate::game::navigations::{NavRequest, Navigation, NavigationMoveTo, Navigations};
use crate::game::wares::Cargo;
use std::borrow::{Borrow, BorrowMut};

pub struct CommandMineSystem;

#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    locations_sector_id: ReadStorage<'a, LocationSector>,
    commands_mine: WriteStorage<'a, CommandMine>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: ReadStorage<'a, Cargo>,
    navigation: ReadStorage<'a, Navigation>,
    action_extract: ReadStorage<'a, ActionMine>,
    action_request: WriteStorage<'a, ActionRequest>,
}

impl<'a> System<'a> for CommandMineSystem {
    type SystemData = CommandMineData<'a>;

    fn run(&mut self, mut data: CommandMineData) {
        trace!("running");

        let sector_index = data.sector_index.borrow();
        // search extractable
        let mut extractables = vec![];

        for (entity, _) in (&data.entities, &data.extractables).join() {
            extractables.push(entity);
        }

        for (entity, command, sector_id, cargo, nav, mining) in (
            &data.entities,
            &mut data.commands_mine,
            &data.locations_sector_id,
            &data.cargos,
            data.navigation.maybe(),
            data.action_extract.maybe(),
        )
            .join()
        {

            let action = execute(sector_index, entity, command, sector_id.sector_id, cargo, nav.is_some(),  mining.is_some(),None);

            match action {
                ExecuteResult::Move { target_id } => {
                    data.nav_request.borrow_mut().insert(entity, NavRequest::MoveToTarget {
                        target_id,
                    });
                },
                ExecuteResult::MoveAndDock { target_id } => {
                    data.nav_request.borrow_mut().insert(entity, NavRequest::MoveAndDockAt{
                        target_id,
                    });
                },
                ExecuteResult::Extract { target_id } => {
                    data.action_request.borrow_mut().insert(entity, ActionRequest(Action::Extract {
                        target_id: target_id
                    }));
                },
                ExecuteResult::NoAction => {},
            }
        }
    }
}


struct ExecuteResult {
    id: ObjId,
    action_request: Option<ActionRequest>,
    nav_request: Option<NavRequest>,
}

enum ExecuteResult {
    Move { target_id: ObjId },
    MoveAndDock { target_id: ObjId },
    Extract { target_id: ObjId },
    NoAction,
}

fn execute(
    sectors_index: &EntityPerSectorIndex,
    entity: ObjId,
    command: &mut CommandMine,
    sector_id: SectorId,
    cargo: Cargo,
    has_navigation: bool,
    is_extracting: bool,
    docket_at: Option<ObjId>,
) -> ExecuteResult {
    // do nothing if is doing navigation
    if has_navigation {
        return ExecuteResult::NoAction;
    }

    // deliver for full cargo
    if cargo.is_full() {
        // find deliver target if not defined
        let target_id = match command.deliver_target_id {
            Some(id) => id,
            None => search_deliver_target(sectors_index, entity, command,  sector_id),
        };

        // if is docked at deliver
        if docket_at == Some(target_id) {
            // deliver
            unimplemented!()
        } else {
            ExecuteResult::MoveAndDock { target_id }
        }
        // continue to mine
    } else {
        // mine for non full cargo
        let target_id = match command.mine_target_id {
            Some(id) => id,
            None => search_mine_target(sectors_index, entity, command, sector_id),
        };

        // navigate to mine
        if sectors_index.is_near(entity, target_id) {
            ExecuteResult::Extract { target_id }
        } else{
            // move to target
            ExecuteResult::Move{ target_id }
        }
    }
}

fn search_mine_target(
    sectors_index: &EntityPerSectorIndex,
    entity: Entity,
    command: &mut CommandMine,
    sector_id: SectorId,
) -> ObjId {
    // find nearest extractable
    let candidates = sectors_index.list_extractables();
    let target_id = candidates.iter().next().unwrap();

    command.mine_target_id = Some(target_id.1);
    target_id.1
}

fn search_deliver_target(
    sectors_index: &EntityPerSectorIndex,
    entity: Entity,
    command: &mut CommandMine,
    sector_id: SectorId,
) -> ObjId {
    // find nearest deliver
    let candidates = sectors_index.list_stations();
    let target_id = candidates.iter().next().unwrap();
    command.deliver_target_id = Some(target_id.1);
    target_id.1
}

/// For miners without target, search nearest one and create a navigation request
pub struct SearchMineTargetsSystem;

#[derive(SystemData)]
pub struct SearchMineTargetsData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    // TODO: what if is not in a sector?
    locations_sector_id: ReadStorage<'a, LocationSector>,
    commands_mine: WriteStorage<'a, CommandMine>,
    commands_mine_target: WriteStorage<'a, CommandMineTargetState>,
    nav_request: WriteStorage<'a, NavRequest>,
}

impl<'a> System<'a> for SearchMineTargetsSystem {
    type SystemData = SearchMineTargetsData<'a>;

    fn run(&mut self, mut data: SearchMineTargetsData) {
        // search extractable
        let mut extractables = vec![];

        trace!("running");

        for (entity, _) in (&data.entities, &data.extractables).join() {
            extractables.push(entity);
        }

        let mut selected = vec![];

        for (entity, _, _, location_sector_id) in (
            &data.entities,
            &data.commands_mine,
            !&data.commands_mine_target,
            &data.locations_sector_id,
        )
            .join()
        {
            let sector_id = location_sector_id.sector_id;

            // TODO: search for nearest
            let target: &ObjId = extractables.iter().next().unwrap();

            // set mine command
            let command = CommandMineTargetState {
                target_id: target.clone(),
            };

            let request = NavRequest::MoveToTarget {
                target_id: target.clone(),
            };

            selected.push((entity, command, request));
        }

        for (entity, state, request) in selected {
            info!(
                "{:?} setting mine target to {:?} and request navigation {:?}",
                entity, state, request
            );
            data.commands_mine_target.insert(entity, state).unwrap();
            data.nav_request.insert(entity, request).unwrap();
        }
    }
}

pub struct MineTargetSystem;

#[derive(SystemData)]
pub struct MineTargetData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    commands_mine: ReadStorage<'a, CommandMine>,
    commands_mine_target: ReadStorage<'a, CommandMineTargetState>,
    nav: ReadStorage<'a, Navigation>,
    nav_request: WriteStorage<'a, NavRequest>,
    location_space: ReadStorage<'a, LocationSpace>,
    //    action_extract: WriteStorage<'a, ActionExtract>,
}

impl<'a> System<'a> for MineTargetSystem {
    type SystemData = MineTargetData<'a>;

    fn run(&mut self, mut data: MineTargetData) {
        trace!("running");

        for (entity, _, mine_target, _) in (
            &data.entities,
            &data.commands_mine,
            &data.commands_mine_target,
            !&data.nav,
        )
            .join()
        {
            //            let target_pos = data.location_space.borrow().get(mine_target.target_id);

            // if is near, mine

            // else move to targeat
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::locations::LocationSector;
    use crate::game::sectors::test_scenery;
    use crate::game::sectors::test_scenery::{SECTOR_0, SECTOR_1};
    use crate::game::wares::WareId;
    use crate::test::test_system;
    use specs::DispatcherBuilder;

    struct SceneryRequest {}

    struct SceneryResult {
        miner: ObjId,
        asteroid: ObjId,
    }

    const WARE_0: WareId = WareId(0);
    const EXTRACTABLE: Extractable = Extractable {
        ware_id: WARE_0,
        time: DeltaTime(1.0),
    };

    fn setup_scenery(world: &mut World) -> SceneryResult {
        let asteroid = world
            .create_entity()
            .with(LocationSector {
                sector_id: SECTOR_1,
            })
            .with(EXTRACTABLE)
            .build();

        let miner = world
            .create_entity()
            .with(LocationSector {
                sector_id: SECTOR_0,
            })
            .with(CommandMine::new())
            .build();

        // TODO: use index
        //        let mut entitys_per_sector = EntityPerSectorIndex::new();
        //        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        //        world.insert(entitys_per_sector);

        SceneryResult { miner, asteroid }
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(SearchMineTargetsSystem, |world| setup_scenery(world));

        let command_storage = world.read_component::<CommandMineTargetState>();
        let command = command_storage.get(scenery.miner);
        match command {
            Some(command) => {
                assert_eq!(command.target_id, scenery.asteroid);
            }
            None => {
                panic!("miner has no commandmine");
            }
        }

        let request_storage = world.read_storage::<NavRequest>();
        match request_storage.get(scenery.miner) {
            Some(NavRequest::MoveToTarget { target_id: target }) => {
                assert_eq!(target.clone(), scenery.asteroid);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_command_mine_should_mine() {
        let (world, scenery) = test_system(SearchMineTargetsSystem, |world| {
            let asteroid = world
                .create_entity()
                .with(LocationSector {
                    sector_id: SECTOR_0,
                })
                .with(LocationSpace {
                    pos: V2::new(0.0, 0.0),
                })
                .with(EXTRACTABLE)
                .build();

            let miner = world
                .create_entity()
                .with(LocationSector {
                    sector_id: SECTOR_0,
                })
                .with(LocationSpace {
                    pos: V2::new(0.01, 0.0),
                })
                .with(CommandMine::new())
                .build();

            SceneryResult { miner, asteroid }
        });

        //        world.read_component::<CommandMine>
    }
}
