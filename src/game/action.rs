use crate::utils::{V2, Position, Log, Seconds};
use crate::game::objects::{ObjRepo, Location, ObjId};
use crate::game::sectors::SectorRepo;
use crate::game::Tick;
use std::collections::HashMap;

#[derive(Clone,Debug)]
pub enum Action {
    Idle,
    Undock,
    Fly { to: Position },
    Jump,
    Mine,
}

const MIN_DISTANCE: f32 = 0.01;
const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

struct State {
    action: Action,
    action_delay: Option<Seconds>,
}

impl State {
    fn new() -> Self {
        State {
            action: Action::Idle,
            action_delay: None,
        }
    }
}

pub struct Actions {
    states: HashMap<ObjId, State>
}

impl Actions {
    pub fn new() -> Self {
        Actions {
            states: HashMap::new(),
        }
    }

    pub fn init(&mut self, obj_id: ObjId) {
        self.states.insert(obj_id, State::new());
    }

    pub fn tick(&mut self, tick: &Tick, objects: &mut ObjRepo, sectors: &SectorRepo) {
        let mut set_actions: Vec<(ObjId, Option<Action>, Option<Location>)> = vec![];

        for obj in objects.list() {
            let state = self.states.get(&obj.id).unwrap();

            match (&state.action, &obj.location) {
                (Action::Idle, _) => {
                    // ignore
                },
                (Action::Undock, Location::Docked { obj_id }) => {
                    let station = objects.get(&obj_id);

                    let (sector_id, pos) = match station.location {
                        Location::Space { sector_id, pos } => (sector_id, pos),
                        _ => panic!("station is not at space")
                    };

                    let new_location = Location::Space {
                        sector_id,
                        pos
                    };

                    set_actions.push((obj.id, Some(Action::Idle), Some(new_location)));
                },
                (Action::Undock, Location::Space { .. }) => {
                    set_actions.push((obj.id, Some(Action::Idle), None));
                },
                (Action::Fly { to }, Location::Space { sector_id, pos}) => {
                    let delta   = to.sub(pos);
                    // delta == zero can cause length sqr NaN
                    let length_sqr = delta.length_sqr();
                    let norm = delta.div(length_sqr.sqrt());
                    let speed = obj.max_speed.unwrap();
                    let mov = norm.mult(speed.0 * tick.delta_time.0);

//                    Log::debug("actions", &format!("{:?} {:?} {:?} {:?} {:?} {:?}", pos, to, delta, length_sqr, norm, mov));

                    let (action, location) =
                        // if current move distance is bigger that distance to arrive, move to the position
                        if length_sqr.is_nan() || length_sqr <= mov.length_sqr() {
                            Log::debug("actions", &format!("{:?} arrive at {:?}", obj.id, to));
                            (
                                Some(Action::Idle),
                                Some(Location::Space {
                                    sector_id: *sector_id,
                                    pos: to.clone()
                                })
                            )
                        } else {
                            let new_position = pos.add(&mov);
                            Log::debug("actions", &format!("{:?} move to {:?}", obj.id, new_position));

                            (
                                None,
                                Some(Location::Space {
                                    sector_id: *sector_id,
                                    pos: new_position
                                })
                            )
                        };

                    set_actions.push((obj.id, action, location));
                },
                (Action::Jump, Location::Space { sector_id, pos}) => {
                    let jump = sectors.find_jump_at(sector_id, pos).unwrap();
                    Log::debug("actions", &format!("{:?} jump to {:?}", obj.id, jump.to));

                    let location = Location::Space {
                        sector_id: jump.to,
                        pos: jump.pos
                    };
                    set_actions.push((obj.id, Some(Action::Idle), Some(location)));

                },
                _ => {
                    Log::warn("actions", &format!("unknown {:?}", obj));
                }
            }
        }

        for (obj_id, action, location) in set_actions {
            action.into_iter().for_each(|action| {
                self.set_action(obj_id, action);
            });

            if let Some(location) = location {
                objects.set_location(obj_id, location);
            }
        }
    }

    pub fn set_action(&mut self, obj_id: ObjId, action: Action) {
        let mut state = self.get_state_mut(&obj_id);
        Log::info("actions", &format!("set action {:?}: {:?}", obj_id, action));
        state.action = action;
    }

    pub fn get_action(&self, obj_id: &ObjId) -> &Action {
        &self.states.get(&obj_id).unwrap().action
    }

    fn get_state_mut(&mut self, id: &ObjId) -> &mut State {
        self.states.get_mut(id).unwrap()
    }
}
