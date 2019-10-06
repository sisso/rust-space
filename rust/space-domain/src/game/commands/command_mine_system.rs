use specs::prelude::*;
use shred::{Read, ResourceId, SystemData, World, Write};
use specs_derive::*;

use super::*;
use crate::game::locations::{LocationDock, EntityPerSectorIndex};
use std::borrow::{Borrow, BorrowMut};
use crate::game::extractables::Extractable;

#[derive(SystemData)]
pub struct SearchMineTargetsData<'a> {
    entities: Entities<'a>,
    navigation_index: Read<'a, EntityPerSectorIndex>,
    locations: ReadStorage<'a, LocationSpace>,
    extractables: ReadStorage<'a, Extractable>,
    states: WriteStorage<'a, MineState>,
}

struct SearchMineTargetsSystem;

impl<'a> System<'a> for SearchMineTargetsSystem {
    type SystemData = SearchMineTargetsData<'a>;

    fn run(&mut self, mut data: SearchMineTargetsData) {
        use specs::Join;

        let nav = data.navigation_index.borrow();
        let mut inserts = vec![];

        for (entity, location, ()) in (&data.entities, &data.locations, !&data.states).join() {
            let candidates = match nav.index_extractables.get(&location.sector_id) {
                Some(candidates) if candidates.len() > 0 => candidates,
                _ => continue,
            };

            // TODO: search for nearest
            let first = candidates.first().unwrap();

            let state = MineState {
                mining: false,
                target_obj_id: *first
            };

            inserts.push((entity, state));
        }

        for (entity, state) in inserts {
            data.states.insert(entity, state).unwrap();
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
    mine_states: WriteStorage<'a, MineState>,
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
