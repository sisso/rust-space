use super::sectors::*;
use super::Tick;
use super::action::*;
use super::objects::*;
use crate::utils::*;
use std::collections::HashMap;
use crate::game::locations::{Location, Locations, LocationSpace};

#[derive(Debug, Clone)]
pub enum Command {
    Idle,
    Mine,
}

struct MineState {
    target_obj_id: ObjId,
}

#[derive(Debug)]
enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump { sector_id: SectorId }
}

struct NavigationState {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: V2,
    path: Vec<NavigationStateStep>
}

struct State {
    command: Command,
    mine: Option<MineState>,
    navigation: Option<NavigationState>,
}

impl State {
    fn new() -> Self {
        State {
            command: Command::Idle,
            mine: None,
            navigation: None
        }
    }
}

// TODO: how to remove state on entity removal?
pub struct Commands {
    state: HashMap<ObjId, State>
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            state: HashMap::new()
        }
    }

    pub fn init(&mut self, obj_id: ObjId) {
        Log::info("commands", &format!("init {:?}", obj_id));
        self.state.insert(obj_id, State::new());
    }

    pub fn tick(&mut self, tick: &Tick, objects: &mut ObjRepo, actions: &mut Actions, locations: &Locations, sectors: &SectorRepo) {
        for (obj_id, state) in self.state.iter_mut() {
            let action = actions.get_action(obj_id);
            let location = locations.get_location(obj_id).unwrap();

            match (&state.command, action, location) {
                (Command::Mine, Action::Idle, Location::Docked { .. }) => {
                    actions.set_action(obj_id, Action::Undock);
                },
                (Command::Mine, Action::Idle, Location::Space { sector_id, pos}) => {
                    if state.mine.is_none() {
                        // TODO: unwarp
                        let target = search_mine_target(objects, sectors, obj_id).unwrap();
                        // TODO: unwarp
                        let navigation = find_navigation_to(sectors, locations, &location.as_space(), target.id).unwrap();

                        state.mine = Some(MineState { target_obj_id: target.id });
                        state.navigation = Some(navigation);
                    }

                    let action = navigation_next_action(sectors, obj_id, &mut state.navigation.as_mut().unwrap());
                    actions.set_action(obj_id, action);
                },
                (Command::Mine, Action::Fly { to}, Location::Space { sector_id, pos}) => {
                    // ignore
                },
                (Command::Mine, Action::Undock, Location::Docked { .. }) => {
                    // ignore
                },
                (Command::Idle, Action::Idle, _) => {
                    // ignore
                },
                (Command::Idle, _, _) => {
                    actions.set_action(obj_id, Action::Idle);
                },
                (a, b, c) => {
                    Log::warn("command", &format!("unknown {:?}", obj_id));
                }
            }
        }
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        let mut state = self.get_state_mut(&obj_id);
        Log::info("commands", &format!("set command {:?}: {:?}", obj_id, command));
        state.command = command;
    }

    fn get_state_mut(&mut self, id: &ObjId) -> &mut State {
        self.state.get_mut(id).unwrap()
    }

    fn get_state(&self, id: &ObjId) -> &State {
        self.state.get(&id).unwrap()
    }
}


fn search_mine_target<'a>(objects: &'a ObjRepo, sectors: &SectorRepo, obj_id: &ObjId) -> Option<&'a Obj> {
    // search minerable
    let candidates =
        objects.list().find(|obj| {
            obj.extractable.is_some()
        });

    // collect params
    candidates
}

// TODO: support movable objects
// TODO: support docked objects
fn find_navigation_to(sectors: &SectorRepo, locations: &Locations, from: &LocationSpace, to_obj_id: ObjId) -> Option<NavigationState> {
    // collect params
    let location = locations.get_location(&to_obj_id).unwrap();
    let target_pos= location.as_space();
    let path = find_path(sectors, from, &target_pos);

    Some(
        NavigationState {
            target_obj_id: to_obj_id,
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

fn navigation_next_action(sectors: &SectorRepo, object_id: &ObjId, navigation: &mut NavigationState) -> Action {
    match navigation.path.pop() {
        Some(NavigationStateStep::MoveTo { pos}) => {
            Action::Fly { to: pos }
        },
        Some(NavigationStateStep::Jump { .. }) => {
            Action::Jump
        },
        None => Action::Idle,
    }
}
