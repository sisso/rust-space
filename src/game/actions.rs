use crate::utils::{V2, Position, Log, Seconds};
use super::objects::{ObjRepo, ObjId};
use super::locations::{Location};
use super::sectors::SectorRepo;
use super::Tick;
use std::collections::HashMap;
use crate::game::locations::Locations;
use crate::game::extractables::Extractables;
use crate::game::wares::Cargos;

mod executor_action_dockundock;
mod executor_action_jump;
mod executor_action_fly;
mod executor_action_mine;

#[derive(Clone,Debug)]
pub enum Action {
    Idle,
    Undock,
    Fly { to: Position },
    Jump,
    Mine { target: ObjId },
}

pub const MIN_DISTANCE: f32 = 0.01;
pub const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

pub struct ActionState {
    pub action: Action,
    pub action_delay: Option<Seconds>,
}

impl ActionState {
    fn new() -> Self {
        ActionState {
            action: Action::Idle,
            action_delay: None,
        }
    }
}

pub struct Actions {
    states: HashMap<ObjId, ActionState>
}

impl Actions {
    pub fn new() -> Self {
        Actions {
            states: HashMap::new(),
        }
    }

    pub fn init(&mut self, obj_id: ObjId) {
        Log::info("actions", &format!("init {:?}", obj_id));
        self.states.insert(obj_id, ActionState::new());
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut ActionState)> + 'a {
        self.states.iter_mut()
    }

    pub fn execute(&mut self, tick: &Tick, sectors: &SectorRepo, locations: &mut Locations, extractables: &Extractables, cargos: &mut Cargos) {
        executor_action_dockundock::execute(self, locations);
        executor_action_jump::execute(self, locations, sectors);
        executor_action_fly::execute(tick, self, locations, sectors);
        executor_action_mine::execute(tick, self, locations, extractables, cargos);
    }

    pub fn set_action(&mut self, obj_id: &ObjId, action: Action) {
        let mut state = self.states.get_mut(&obj_id).expect(&format!("{:?} action not found", obj_id));
        Log::info("actions", &format!("set action {:?}: {:?}", obj_id, action));
        state.action = action;
    }

    pub fn get_action(&self, obj_id: &ObjId) -> &Action {
        &self.states.get(&obj_id).unwrap().action
    }
}
