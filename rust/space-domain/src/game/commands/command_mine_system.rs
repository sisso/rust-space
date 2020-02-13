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
use crate::game::locations::{EntityPerSectorIndex, Location};
use crate::game::navigations::{NavRequest, Navigation, NavigationMoveTo, Navigations};
use crate::game::wares::Cargo;
use std::borrow::{Borrow, BorrowMut};

pub struct CommandMineSystem;

#[derive(SystemData)]
pub struct CommandMineData<'a> {
    entities: Entities<'a>,
    extractables: ReadStorage<'a, Extractable>,
    locations: ReadStorage<'a, Location>,
    commands_mine: WriteStorage<'a, CommandMine>,
    nav_request: WriteStorage<'a, NavRequest>,
    sector_index: Read<'a, EntityPerSectorIndex>,
    cargos: WriteStorage<'a, Cargo>,
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

        let mut cargo_transfers = vec![];

        for (entity, command, cargo, nav, mining, location) in (
            &data.entities,
            &mut data.commands_mine,
            &data.cargos,
            data.navigation.maybe(),
            data.action_extract.maybe(),
            &data.locations,
        )
            .join()
        {
            let result = execute(
                sector_index,
                entity,
                command,
                location,
                cargo,
                nav.is_some(),
                mining.is_some(),
                None,
                |id| {
                    let locations = data.locations.borrow();
                    Locations::resolve_space_position(locations, id)
                        .map(|i| i.sector_id)
                },
                |id| {
                    let locations = data.locations.borrow();
                    locations.get(id).cloned()
                },
            );

            if let Some(nav_request) = result.nav_request {
                data.nav_request.borrow_mut().insert(entity, nav_request);
            }

            if let Some(action_request) = result.action_request {
                data.action_request.borrow_mut().insert(entity, action_request);
            }

            if let Some(transfer_target_id) = result.transfer_cargo {
                cargo_transfers.push((entity, transfer_target_id));
            }
        }

        // transfer all cargos
        let cargos = data.cargos.borrow_mut();
        for (from_id, to_id) in cargo_transfers {
            Cargos::move_all(cargos, from_id, to_id);
        }
    }
}


struct ExecuteResult {
    id: ObjId,
    action_request: Option<ActionRequest>,
    nav_request: Option<NavRequest>,
    transfer_cargo: Option<ObjId>,
}

fn execute<F1, F2>(
    sectors_index: &EntityPerSectorIndex,
    entity: ObjId,
    command: &mut CommandMine,
    location: &Location,
    cargo: &Cargo,
    has_navigation: bool,
    is_extracting: bool,
    docket_at: Option<ObjId>,
    resolve_sector_id: F1,
    resolve_location: F2,
) -> ExecuteResult
    where
        F1: Fn(ObjId) -> Option<SectorId>,
        F2: Fn(ObjId) -> Option<Location>,
{


    let mut result = ExecuteResult {
        id: entity,
        nav_request: None,
        action_request: None,
        transfer_cargo: None,
    };

    debugf!("Here we are");

    // do nothing if is doing navigation
    if has_navigation {
        return result;
    }

    // deliver for full cargo
    if cargo.is_full() {
        // find deliver target if not defined
        let target_id = match command.deliver_target_id {
            Some(id) => id,
            None => {
                let sector_id = resolve_sector_id(entity).unwrap();
                search_deliver_target(sectors_index, entity, command, sector_id)
            },
        };

        // if is docked at deliver
        if docket_at == Some(target_id) {
            // deliver
            result.transfer_cargo = Some(target_id);
        } else {
            result.nav_request = Some(NavRequest::MoveAndDockAt { target_id });
        }
        // continue to minbenefits oe
    } else {
        // mine for non full cargo
        let target_id = match command.mine_target_id {
            Some(id) => id,
            None => {
                let sector_id = resolve_sector_id(entity).unwrap();
                search_mine_target(sectors_index, entity, command, sector_id)
            },
        };

        // navigate to mine
        let target_location = resolve_location(target_id).unwrap();
        if Locations::is_near(location, &target_location) {
            result.action_request = Some(ActionRequest(Action::Extract { target_id }));
        } else {
            // move to target
            result.nav_request = Some(NavRequest::MoveToTarget { target_id });
        }
    }

    return result;
}

fn search_mine_target(
    sectors_index: &EntityPerSectorIndex,
    entity: Entity,
    command: &mut CommandMine,
    sector_id: SectorId,
) -> ObjId {
    // find nearest extractable
    let candidates = sectors_index.search_nearest_extractable(sector_id);
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


#[cfg(test)]
mod test {
    use super::*;
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
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_1,
            })
            .with(EXTRACTABLE)
            .build();

        let miner = world
            .create_entity()
            .with(Location::Space {
                pos: V2::new(0.0, 0.0),
                sector_id: SECTOR_0,
            })
            .with(CommandMine::new())
            .with(Cargo::new(10.0))
            .build();

        // TODO: use index
        let mut entitys_per_sector = EntityPerSectorIndex::new();
        entitys_per_sector.add_extractable(SECTOR_1, asteroid);
        world.insert(entitys_per_sector);

        SceneryResult { miner, asteroid }
    }

    #[test]
    fn test_command_mine_should_setup_navigation() {
        let (world, scenery) = test_system(CommandMineSystem, |world| setup_scenery(world));

        let command_storage = world.read_component::<CommandMine>();
        let command = command_storage.get(scenery.miner);
        match command {
            Some(command) => {
                assert_eq!(command.mine_target_id, Some(scenery.asteroid));
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
        let (world, scenery) = test_system(CommandMineSystem, |world| {
            let asteroid = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(0.0, 0.0),
                    sector_id: SECTOR_0,
                })
                .with(EXTRACTABLE)
                .build();

            let miner = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(0.01, 0.0),
                    sector_id: SECTOR_0,
                })
                .with(CommandMine::new())
                .build();

            SceneryResult { miner, asteroid }
        });

        //        world.read_component::<CommandMine>
    }
}
