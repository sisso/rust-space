use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use super::super::sectors::*;
use super::super::Tick;
use super::*;
use super::super::objects::*;
use crate::utils::*;
use crate::game::locations::{Location, Locations, LocationSpace};
use crate::game::extractables::Extractables;

pub fn execute(tick: &Tick, commands: &mut Commands, extractables: &Extractables, actions: &mut Actions, locations: &Locations, sectors: &SectorRepo) {
    for (obj_id, state) in commands.list_mut() {
        match state.command {
            Command::Mine => {
                do_command_mine(extractables, actions, locations, sectors, obj_id, state);
            },
            _ => {},
        }
    }
}

fn do_command_mine(extractables: &Extractables, actions: &mut Actions, locations: &Locations, sectors: &SectorRepo, obj_id: &ObjId, state: &mut CommandState) -> () {
    let action = actions.get_action(obj_id);
    let location = locations.get_location(obj_id).unwrap();

    match (action, location) {
        (Action::Idle, Location::Docked { .. }) => {
            actions.set_action(obj_id, Action::Undock);
        },
        (Action::Idle, Location::Space { sector_id, pos }) => {
            do_command_mine_idle(extractables, actions, locations, sectors, obj_id, state, location)
        },
        (Action::Fly { to }, Location::Space { sector_id, pos }) => {
            // ignore
        },
        (Action::Undock, Location::Docked { .. }) => {
            // ignore
        },
        (a, b) => {
            Log::warn("command", &format!("unknown {:?}", obj_id));
        }
    }
}

fn do_command_mine_idle(extractables: &Extractables, actions: &mut Actions, locations: &Locations, sectors: &SectorRepo, obj_id: &ObjId, state: &mut CommandState, location: &Location) -> () {
    if state.mine.is_none() {
        set_mine_state_nearest_target(extractables, locations, sectors, obj_id, state, location);
        Log::info("commands", &format!("{:?} creating mining state {:?}", obj_id, state.mine));
    }

    let mine_state = state.mine.as_mut().unwrap();
    if mine_state.mining {
        return;
    }

    if state.navigation.iter().any(|i| i.is_complete()) {
        Log::info("commands", &format!("{:?} arrive to mine location", obj_id));
        mine_state.mining = true;

        actions.set_action(obj_id, Action::Mine {
            target: mine_state.target_obj_id
        });
    } else {
        state.navigation.iter_mut().for_each(|mut i| {
            let action = i.navigation_next_action();
            actions.set_action(obj_id, action);
        });
    }
}

fn set_mine_state_nearest_target(extractables: &Extractables, locations: &Locations, sectors: &SectorRepo, obj_id: &ObjId, state: &mut CommandState, location: &Location) {
    // TODO: unwarp
    let target = search_mine_target(extractables, sectors, obj_id).unwrap();
    // TODO: unwarp
    let navigation = find_navigation_to(sectors, locations, &location.as_space(), &target).unwrap();

    state.mine = Some(MineState {
        mining: false,
        target_obj_id: target
    });
    state.navigation = Some(navigation);
}

fn search_mine_target(extractables: &Extractables, sectors: &SectorRepo, obj_id: &ObjId) -> Option<ObjId> {
    // TODO: search nearest
    let candidates = extractables.list().find(|_| true);
    candidates.map(|i| i.clone())
}

// TODO: support movable objects
// TODO: support docked objects
fn find_navigation_to(sectors: &SectorRepo, locations: &Locations, from: &LocationSpace, to_obj_id: &ObjId) -> Option<NavigationState> {
    // collect params
    let location = locations.get_location(&to_obj_id).unwrap();
    let target_pos= location.as_space();
    let path = find_path(sectors, from, &target_pos);

    Some(
        NavigationState {
            target_obj_id: *to_obj_id,
            target_sector_id: target_pos.sector_id,
            target_position: target_pos.pos,
            path: path
        }
    )
}

fn find_path(sectors: &SectorRepo, from: &LocationSpace, to: &LocationSpace) -> Vec<NavigationStateStep> {
    let mut path: Vec<NavigationStateStep> = vec![];

    let mut current = from.clone();

    loop {
        if current.sector_id == to.sector_id {
            path.push(NavigationStateStep::MoveTo { pos: to.pos });
            break;
        } else {
            let current_sector = sectors.get(&current.sector_id);
            let jump = current_sector.jumps.iter().find(|jump| {
                jump.to == to.sector_id
            }).unwrap();

            path.push(NavigationStateStep::MoveTo { pos: jump.pos });
            path.push(NavigationStateStep::Jump { sector_id: jump.to });

            let arrival_sector = sectors.get(&jump.to);
            let arrival_jump = arrival_sector.jumps.iter().find(|jump| {
                jump.to == current_sector.id
            }).unwrap();
            let arrival_position = arrival_jump.pos;

            current = LocationSpace {
                sector_id: jump.to,
                pos: arrival_position
            }
        }
    }

    path.reverse();

    Log::debug("command.find_path", &format!("from {:?} to {:?}: {:?}", from, to, path));

    path
}
