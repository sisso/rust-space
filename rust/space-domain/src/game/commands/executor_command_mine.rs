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
use crate::game::wares::Cargos;
use std::collections::VecDeque;

pub fn execute(tick: &Tick,
               commands: &mut Commands,
               objects: &ObjRepo,
               extractables: &Extractables,
               actions: &mut Actions,
               locations: &Locations,
               sectors: &Sectors,
               cargos: &mut Cargos) {

    for (obj_id, state) in commands.list_mut() {
        match state.command {
            Command::Mine => {
                do_command_mine(objects, extractables, actions, locations, sectors, obj_id, state, cargos);
            },
            _ => {},
        }
    }
}

fn do_command_mine(
    objects: &ObjRepo,
    extractables: &Extractables,
    actions: &mut Actions,
    locations: &Locations,
    sectors: &Sectors,
    obj_id: &ObjId,
    state: &mut CommandState,
    cargos: &mut Cargos
) -> () {

    let action = actions.get_action(obj_id);

    let location = match locations.get_location(obj_id) {
        Some(location) => location,
        None => {
            Log::warn("executor_command_mine", &format!("{:?} has no location", obj_id));
            return;
        }
    };

    let is_cargo_full = cargos.get_cargo(obj_id).map(|i| i.is_full()).unwrap_or(false);
    let is_mining = state.mine.as_ref().map(|i| i.mining).unwrap_or(false);

    match (action, location) {
        (Action::Idle, Location::Docked { docked_id: target_id }) if is_cargo_full => {
            Log::info("executor_command_mine", &format!("{:?} deliver cargo", obj_id));
            cargos.move_all(obj_id, target_id);
        },
        (Action::Idle, Location::Docked { .. }) => {
            actions.set_action(obj_id, Action::Undock);
        },
        (Action::Idle, Location::Space { .. }) if is_cargo_full => {
            execute_mine_deliver_resources(
                objects,
                actions,
                locations,
                sectors,
                obj_id,
                state,
                location,
                cargos
            );
        },
        (Action::Idle, Location::Space { .. }) if is_mining => {
            Log::warn("executor_command_mine", &format!("{:?} unexpected state, action is idle and mining state is mining", obj_id));
        },
        (Action::Idle, Location::Space { .. }) => {
            execute_mine_idle(extractables, actions, locations, sectors, obj_id, state, location, cargos)
        },
        (Action::Fly { to }, Location::Space { sector_id, pos }) => {
            // ignore
        },
        (Action::Undock, Location::Docked { .. }) => {
            // ignore
        },
        (Action::Mine { .. }, _) => {
            // ignore
        },
        (a, b) => {
            Log::warn("executor_command_mine", &format!("unknown {:?}", obj_id));
        }
    }
}

fn execute_mine_idle(extractables: &Extractables,
                     actions: &mut Actions,
                     locations: &Locations,
                     sectors: &Sectors,
                     obj_id: &ObjId,
                     state: &mut CommandState,
                     location: &Location,
                     cargos: &mut Cargos) {

    if state.mine.is_none() {
        // TODO: unwarp
        let target = search_mine_target(extractables, sectors, obj_id).unwrap();
        // TODO: unwarp
        let navigation = find_navigation_to(sectors, locations, &location.get_space(), &target).unwrap();

        state.clear();

        state.mine = Some(MineState {
            mining: false,
            target_obj_id: target
        });

        state.navigation = Some(navigation);

        Log::info("executor_command_mine", &format!("{:?} set mining state {:?}, navigation {:?}", obj_id, state.mine, state.navigation));
    }

    let nav = state.navigation.as_mut().unwrap();
    if nav.is_complete() {
        Log::info("executor_command_mine", &format!("{:?} arrive to mine location", obj_id));

        let mine_state = state.mine.as_mut().unwrap();
        mine_state.mining = true;
        actions.set_action(obj_id, Action::Mine {
            target: mine_state.target_obj_id
        });
    } else {
        let action = nav.navigation_next_action();
        actions.set_action(obj_id, action);
    }
}

fn search_mine_target(extractables: &Extractables, sectors: &Sectors, obj_id: &ObjId) -> Option<ObjId> {
    // TODO: search nearest
    let candidates = extractables.list().find(|_| true);
    candidates.map(|i| i.clone())
}

// TODO: support movable objects
// TODO: support docked objects
fn find_navigation_to(sectors: &Sectors, locations: &Locations, from: &LocationSpace, to_obj_id: &ObjId) -> Option<NavigationState> {
    // collect params
    let location = locations.get_location(&to_obj_id).unwrap();
    let target_pos= location.get_space();
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

fn find_path(sectors: &Sectors, from: &LocationSpace, to: &LocationSpace) -> VecDeque<NavigationStateStep> {
    let mut path: VecDeque<NavigationStateStep> = VecDeque::new();

    let mut current = from.clone();

    loop {
        if current.sector_id == to.sector_id {
            path.push_back(NavigationStateStep::MoveTo { pos: to.pos });
            break;
        } else {
            let current_sector = sectors.get(&current.sector_id);
            let jump = current_sector.jumps.iter().find(|jump| {
                jump.to == to.sector_id
            }).unwrap();

            path.push_back(NavigationStateStep::MoveTo { pos: jump.pos });
            path.push_back(NavigationStateStep::Jump { sector_id: jump.to });

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

    Log::debug("executor_command_mine", &format!("navigation path from {:?} to {:?}: {:?}", from, to, path));

    path
}

fn execute_mine_deliver_resources(
    objects: &ObjRepo,
    actions: &mut Actions,
    locations: &Locations,
    sectors: &Sectors,
    obj_id: &ObjId,
    state: &mut CommandState,
    location: &Location,
    cargos: &mut Cargos
) {
    if state.deliver.is_none() {
        let target = match search_deliver_target(objects, obj_id) {
            Some(target) => target,
            None => {
                Log::warn("executor_command_mine", &format!("{:?} fail to find deliver target", obj_id));
                return;
            },
        };

        state.clear();

        state.deliver = Some(
          DeliverState {
              target_obj_id: target,
          }
        );

        state.navigation =
            find_navigation_to(sectors, locations, &location.get_space(), &target)
                .map(|mut nav| {
                    nav.append_dock_at(target);
                    nav
                });

        Log::info("executor_command_mine", &format!("{:?} set deliver state {:?}, navigation {:?}", obj_id, state.deliver, state.navigation));
    }

    // println!("{:?}", state);
    let nav = state.navigation.as_mut().unwrap();
    if nav.is_complete() {
        Log::info("executor_command_mine", &format!("{:?} arrive to deliver location", obj_id));
    } else {
        let action = nav.navigation_next_action();
        actions.set_action(obj_id, action);
    }
}

fn search_deliver_target(objects: &ObjRepo, obj_id: &ObjId) -> Option<ObjId> {
    objects
        .list()
        .find(|obj| obj.has_dock)
        .map(|i| i.id)
}
