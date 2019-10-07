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
use crate::game::locations::{LocationDock, EntityPerSectorIndex};
use std::borrow::{Borrow, BorrowMut};
use crate::game::extractables::Extractable;
use crate::game::navigations::{Navigation, NavigationMoveTo};

// TODO: what about docked entities?
pub struct SearchMineTargetsSystem;

#[derive(SystemData)]
pub struct SearchMineTargetsData<'a> {
    entities: Entities<'a>,
    navigation_index: Read<'a, EntityPerSectorIndex>,
    locations_dock: ReadStorage<'a, LocationDock>,
    locations_space: ReadStorage<'a, LocationSpace>,
    commands: ReadStorage<'a, HasCommand>,
    commands_mine: WriteStorage<'a, CommandMine>,
}

impl<'a> System<'a> for SearchMineTargetsSystem {
    type SystemData = SearchMineTargetsData<'a>;

    fn run(&mut self, mut data: SearchMineTargetsData) {
        use specs::Join;
        use specs::hibitset::BitSetLike;

        let nav = data.navigation_index.borrow();
        let mut selected = vec![];

        for (entity, command, _, docked, space) in (&data.entities, &data.commands, !&data.commands_mine, data.locations_dock.maybe(), data.locations_space.maybe()).join() {
            match command {
                HasCommand::Mine => {},
                _ => continue,
            }

            let sector_id = match (docked, space) {
                (Some(docked), None) => {
                    // TODO: maybe add sector? even when it is docked?
                    let location = data.locations_space.get(docked.docked_id).unwrap();
                    location.sector_id
                },
                (None, Some(space)) => space.sector_id,
                _ => panic!(),
            };

            // search candidates
            let candidates = match nav.index_extractables.get(&sector_id) {
                Some(candidates) if candidates.len() > 0 => candidates,
                _ => continue,
            };

            // search for nearest?
            let target = candidates.first().unwrap();

            // set mine command
            let command = CommandMine {
                mining: false,
                target_obj_id: target.clone()
            };

            selected.push((entity, command));
        }

        for (entity, state) in selected {
            data.commands_mine.insert(entity, state).unwrap();
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


/// create navigation plans for new miners
///
///
pub struct CreateNavigationSystem;

#[derive(SystemData)]
pub struct CreateNavigationData<'a> {
    entities: Entities<'a>,
    sectors_index: Read<'a, SectorsIndex>,
    commands_mine: ReadStorage<'a, CommandMine>,
    actions_mine: ReadStorage<'a, ActionMine>,
    navigations: WriteStorage<'a, Navigation>,
    navigations_move_to: WriteStorage<'a, NavigationMoveTo>,
}

impl<'a> System<'a> for CreateNavigationSystem {
    type SystemData = CreateNavigationData<'a>;

    fn run(&mut self, mut data: CreateNavigationData) {
        use specs::Join;

        let sector_index = data.sectors_index.borrow();


        for (commands_mine) in (&data.commands_mine, !&data.navigations, !&data.actions_mine).join() {

        }
    }
}
