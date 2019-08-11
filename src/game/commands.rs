use super::sectors::*;
use super::Tick;
use super::actions::*;
use super::objects::*;
use crate::utils::*;
use std::collections::HashMap;
use crate::game::locations::{Location, Locations, LocationSpace};
use crate::game::extractables::Extractables;
use crate::game::wares::Cargos;

mod executor_command_idle;
mod executor_command_mine;

#[derive(Debug, Clone)]
pub enum Command {
    Idle,
    Mine,
}

#[derive(Debug, Clone)]
struct MineState {
    pub mining: bool,
    pub target_obj_id: ObjId,
}

#[derive(Debug, Clone)]
enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump { sector_id: SectorId }
}

#[derive(Debug, Clone)]
struct NavigationState {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: V2,
    path: Vec<NavigationStateStep>
}

impl NavigationState {
    fn is_complete(&self) -> bool {
        self.path.is_empty()
    }

    fn navigation_next_action(&mut self) -> Action {
        match self.path.pop() {
            Some(NavigationStateStep::MoveTo { pos}) => {
                Action::Fly { to: pos }
            },
            Some(NavigationStateStep::Jump { .. }) => {
                Action::Jump
            },
            None => Action::Idle,
        }
    }
}

struct CommandState {
    command: Command,
    mine: Option<MineState>,
    navigation: Option<NavigationState>,
}

impl CommandState {
    fn new() -> Self {
        CommandState {
            command: Command::Idle,
            mine: None,
            navigation: None
        }
    }
}

// TODO: how to remove state on entity removal?
pub struct Commands {
    state: HashMap<ObjId, CommandState>
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            state: HashMap::new()
        }
    }

    pub fn init(&mut self, obj_id: ObjId) {
        Log::info("commands", &format!("init {:?}", obj_id));
        self.state.insert(obj_id, CommandState::new());
    }

    pub fn execute(&mut self, tick: &Tick, extractables: &Extractables, actions: &mut Actions, locations: &Locations, sectors: &SectorRepo, cargos: &mut Cargos) {
        executor_command_idle::execute(self, actions);
        executor_command_mine::execute(tick, self, extractables, actions, locations, sectors, cargos);
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        let mut state = self.get_state_mut(&obj_id);
        Log::info("commands", &format!("set command {:?}: {:?}", obj_id, command));
        state.command = command;
    }



    fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut CommandState)> + 'a {
        self.state.iter_mut()
    }

    fn get_state_mut(&mut self, id: &ObjId) -> &mut CommandState {
        self.state.get_mut(id).unwrap()
    }

    fn get_state(&self, id: &ObjId) -> &CommandState {
        self.state.get(&id).unwrap()
    }
}
