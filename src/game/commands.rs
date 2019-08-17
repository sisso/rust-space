use std::collections::{HashMap, VecDeque};

use crate::game::extractables::Extractables;
use crate::game::locations::{Location, Locations, LocationSpace};
use crate::game::save::{Load, Save, CanSave, CanLoad};
use crate::game::wares::Cargos;
use crate::utils::*;

use super::actions::*;
use super::objects::*;
use super::sectors::*;
use super::Tick;
use super::jsons;
use serde_json::Value;
use crate::game::jsons::JsonValueExtra;

mod executor_command_idle;
mod executor_command_mine;

#[derive(Debug, Clone)]
pub enum Command {
    Idle,
    Mine,
}

#[derive(Debug, Clone)]
struct MineState {
    mining: bool,
    target_obj_id: ObjId,
}

#[derive(Debug, Clone)]
struct DeliverState {
    target_obj_id: ObjId,
}

#[derive(Debug, Clone)]
enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump { sector_id: SectorId },
    Dock { target: ObjId },
}

#[derive(Debug, Clone)]
struct NavigationState {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: V2,
    path: VecDeque<NavigationStateStep>
}

impl NavigationState {
    fn is_complete(&self) -> bool {
        self.path.is_empty()
    }

    fn navigation_next_action(&mut self) -> Action {
        match self.path.pop_front() {
            Some(NavigationStateStep::MoveTo { pos}) => {
                Action::Fly { to: pos }
            },
            Some(NavigationStateStep::Jump { .. }) => {
                Action::Jump
            },
            Some(NavigationStateStep::Dock { target }) => {
                Action::Dock { target }
            },
            None => Action::Idle,
        }
    }

    fn append_dock_at(&mut self, target: ObjId) {
        self.path.push_back(NavigationStateStep::Dock {
            target
        })
    }
}

#[derive(Debug, Clone)]
struct CommandState {
    command: Command,
    mine: Option<MineState>,
    deliver: Option<DeliverState>,
    navigation: Option<NavigationState>,
}

impl CommandState {
    fn new() -> Self {
        CommandState {
            command: Command::Idle,
            mine: None,
            deliver: None,
            navigation: None
        }
    }

    fn clear(&mut self) {
        self.mine = None;
        self.deliver = None;
        self.navigation = None;
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

    pub fn execute(&mut self, tick: &Tick, objects: &ObjRepo, extractables: &Extractables, actions: &mut Actions, locations: &Locations, sectors: &Sectors, cargos: &mut Cargos) {
        executor_command_idle::execute(self, actions);
        executor_command_mine::execute(tick, self, objects, extractables, actions, locations, sectors, cargos);
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        let mut state = self.get_state_mut(&obj_id);
        Log::info("commands", &format!("set command {:?}: {:?}", obj_id, command));
        state.command = command;
    }

    pub fn save(&self, save: &mut impl Save) {
        for (id, state) in self.state.iter() {



        }
    }

    pub fn load(&mut self, load: &mut impl Load) {

    }

    fn list_mut<'a>(&'a mut self) -> impl Iterator<Item=(&ObjId, &mut CommandState)> + 'a {
        self.state.iter_mut()
    }

    fn get_state_mut(&mut self, id: &ObjId) -> &mut CommandState {
        self.state.get_mut(id).unwrap()
    }

    fn get_state(&self, id: &ObjId) -> &CommandState {
        self.state.get(&id).unwrap()
    }
}

impl CanSave for Commands {
    fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (obj_id, state) in self.state.iter() {
            let command = match state.command {
                Command::Idle => "idle",
                Command::Mine => "mine",
            };

            let mine_state: Option<Value> = state.mine.as_ref().map(|mine_state| {
                json!({
                    "mining": mine_state.mining,
                    "target_obj_id": mine_state.target_obj_id.0,
                })
            });

            let deliver_state: Option<Value> = state.deliver.as_ref().map(|state| {
                json!({
                    "target_obj_id": state.target_obj_id.0,
                })
            });

            let navigation_state: Option<Value> = state.navigation.as_ref().map(|state| {
                let path: Vec<Value> = state.path.iter().map(|i| {
                    let (step, pos, sector_id, target) = match i {
                        NavigationStateStep::MoveTo { pos } => {
                            ("moveto", Some(jsons::from_v2(pos)), None, None)
                        },
                        NavigationStateStep::Jump { sector_id } => {
                            ("jump", None, Some(sector_id.0), None)
                        },
                        NavigationStateStep::Dock { target } => {
                            ("dock", None, None, Some(target.0))
                        },
                    };

                    json!({
                        "step": step,
                        "pos": pos,
                        "sector_id": sector_id,
                        "target": target,
                    })
                }).collect();

                json!({
                    "target_obj_id": state.target_obj_id.0,
                    "target_sector_id": state.target_sector_id.0,
                    "target_position": jsons::from_v2(&state.target_position),
                    "path": path,
                })
            });

            save.add(obj_id.0, "command", json!({
                "command": command,
                "mine_state": mine_state,
                "deliver_state": deliver_state,
                "navigation_state": navigation_state,
            }));
        }
    }
}

impl CanLoad for Commands {
    fn load(&mut self, load: &mut impl Load) {
        for (id, state) in load.get_components("command") {
            let command = match state["command"].as_str().unwrap().as_ref() {
                "idle" => Command::Idle,
                "mine" => Command::Mine,
                _      => panic!(),
            };

            let mine = state["mine_state"].as_object().map(|state| {
                MineState {
                    mining: state["mining"].as_bool().unwrap(),
                    target_obj_id: ObjId(state["target_obj_id"].to_u32())
                }
            });

            let deliver = state["deliver_state"].as_object().map(|state| {
                DeliverState {
                    target_obj_id: ObjId(state["target_obj_id"].to_u32())
                }
            });

            let navigation = state["navigation_state"].as_object().map(|state| {
                let path = state["path"].as_array().unwrap().iter().map(|state| {
                    match (
                            state["step"].as_str().unwrap().as_ref(),
                            state["pos"].as_opt(),
                            state["sector_id"].as_u64(),
                            state["target"].as_u64()
                        ) {
                        ("jump", None, Some(sector_id), None) => {
                            NavigationStateStep::Jump {
                                sector_id: SectorId(sector_id as u32)
                            }
                        },
                        ("dock", None, None, Some(target_id)) => {
                            NavigationStateStep::Dock {
                                target: ObjId(target_id as u32)
                            }
                        },
                        _ => panic!("fail to parse navigation path")
                    }
                }).collect();

                NavigationState {
                    target_obj_id: ObjId(state["target_obj_id"].to_u32()),
                    target_sector_id: SectorId(state["target_sector_id"].to_u32()),
                    target_position: state["target_position"].to_v2(),
                    path
                }
            });

            self.state.insert(ObjId(*id), CommandState {
                command,
                mine,
                deliver,
                navigation
            });
        }
    }
}

