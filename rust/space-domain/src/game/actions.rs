use crate::utils::{V2, Position, Seconds};
use super::objects::{ObjRepo, ObjId};
use super::locations::{Location};
use super::sectors::Sectors;
use super::Tick;
use std::collections::HashMap;
use crate::game::locations::Locations;
use crate::game::extractables::Extractables;
use crate::game::wares::Cargos;
use crate::game::save::{Save, Load};
use crate::game::jsons::JsonValueExtra;

mod executor_action_dockundock;
mod executor_action_jump;
mod executor_action_fly;
mod executor_action_mine;

#[derive(Clone,Debug)]
pub enum Action {
    Idle,
    Undock,
    Dock { target: ObjId },
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
        info!("actions", &format!("init {:?}", obj_id));
        self.states.insert(obj_id, ActionState::new());
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut ActionState)> + 'a {
        self.states.iter_mut()
    }

    pub fn execute(&mut self, tick: &Tick, sectors: &Sectors, locations: &mut Locations, extractables: &Extractables, cargos: &mut Cargos) {
        executor_action_dockundock::execute(self, locations);
        executor_action_jump::execute(self, locations, sectors);
        executor_action_fly::execute(tick, self, locations, sectors);
        executor_action_mine::execute(tick, self, locations, extractables, cargos);
    }

    pub fn set_action(&mut self, obj_id: &ObjId, action: Action) {
        let mut state = self.states.get_mut(&obj_id).expect(&format!("{:?} action not found", obj_id));
        info!("actions", &format!("set action {:?}: {:?}", obj_id, action));
        state.action = action;
    }

    pub fn get_action(&self, obj_id: &ObjId) -> &Action {
        &self.states.get(&obj_id).unwrap().action
    }

    pub fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (k,v) in self.states.iter() {
            let (action, target_id, target_pos) =
                match v.action {
                    Action::Idle => ("idle", None, None),
                    Action::Mine { target } => ("mine", Some(target.0), None),
                    Action::Dock { target } => ("dock", Some(target.0), None),
                    Action::Jump => ("jump", None, None),
                    Action::Undock => ("undock", None, None),
                    Action::Fly { to } => ("fly", None, Some((to.x, to.y))),
                };

            save.add(k.0, "action", json!({
                "action": action,
                "target_id": target_id,
                "target_pos": target_pos,
            }));
        }
    }
    pub fn load(&mut self, load: &mut impl Load) {
        for (id, value) in load.get_components("action") {
            let action = value["action"].as_str().unwrap();
            let target_id = value["target_id"].as_u64();
            let target_pos = value["target_pos"].as_v2();

            let action = match (action.as_ref(), target_id, target_pos) {
                ("idle", _, _) => Action::Idle,
                ("mine", Some(target_id), _) => Action::Mine { target: ObjId(target_id as u32) },
                ("dock", Some(target_id), _) => Action::Dock { target: ObjId(target_id as u32) },
                ("jump", _, _) => Action::Jump,
                ("undock", _, _) => Action::Undock,
                ("fly", _, Some(target_pos)) => Action::Fly { to: target_pos },
                _ => panic!("unexpected action")
            };

            self.states.insert(ObjId(*id), ActionState {
                action,
                action_delay: None
            });
        }
    }
}
