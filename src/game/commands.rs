use super::sectors::*;
use super::Tick;
use super::action::*;
use super::objects::*;
use crate::utils::*;
use std::collections::HashMap;
use crate::game::locations::{Location, Locations, LocationSpace};
use crate::game::extractables::Extractables;

#[derive(Debug, Clone)]
pub enum Command {
    Idle,
    Mine,
}

#[derive(Debug, Clone)]
pub struct MineState {
    pub mining: bool,
    pub target_obj_id: ObjId,
}

#[derive(Debug, Clone)]
pub enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump { sector_id: SectorId }
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub target_obj_id: ObjId,
    pub target_sector_id: SectorId,
    pub target_position: V2,
    pub path: Vec<NavigationStateStep>
}

impl NavigationState {
    pub fn is_complete(&self) -> bool {
        self.path.is_empty()
    }
}

pub struct State {
    pub command: Command,
    pub mine: Option<MineState>,
    pub navigation: Option<NavigationState>,
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

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut State)> + 'a {
        self.state.iter_mut()
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
