use crate::utils::{V2, Position, Log, Seconds};
use super::objects::{ObjRepo, ObjId};
use super::locations::{Location};
use super::sectors::SectorRepo;
use super::Tick;
use std::collections::HashMap;
use crate::game::locations::Locations;

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
        Log::info("actions", &format!("init {:?}", obj_id));
        self.states.insert(obj_id, State::new());
    }

    pub fn tick(&mut self, tick: &Tick, sectors: &SectorRepo, locations: &mut Locations) {
        for (obj_id, state) in self.states.iter_mut() {
            let location = locations.get_location(obj_id);
            if location.is_none() {
                continue;
            }
            let location = location.unwrap();

            Log::debug("actions", &format!("process {:?} {:?}", obj_id, location));

            match (&state.action, location) {
                (Action::Idle, _) => {
                    // ignore
                },
                (Action::Undock, Location::Docked { obj_id: station_id }) => {
                    let station_location = locations.get_location(&station_id).unwrap();

                    let (sector_id, pos) = match station_location {
                        Location::Space { sector_id, pos } => (sector_id.clone(), pos.clone()),
                        _ => panic!("station is not at space")
                    };

                    let new_location = Location::Space {
                        sector_id,
                        pos
                    };

                    state.action = Action::Idle;
                    locations.set_location(obj_id, new_location);
                },
                (Action::Undock, Location::Space { .. }) => {
                    state.action = Action::Idle;
                },
                (Action::Fly { to }, Location::Space { sector_id, pos}) => {
                    let delta   = to.sub(pos);
                    // delta == zero can cause length sqr NaN
                    let length_sqr = delta.length_sqr();
                    let speed = locations.get_speed(&obj_id).unwrap();
                    let max_distance = speed.0 * tick.delta_time.0;
                    let norm = delta.div(length_sqr.sqrt());
                    let mov = norm.mult(max_distance);

//                    Log::debug("actions", &format!("pos {:?}, to {:?}, delta {:?}, lensqr {:?}, norm {:?}, mov {:?}", pos, to, delta, length_sqr, norm, mov));

                    // if current move distance is bigger that distance to arrive, move to the position
                    if length_sqr.is_nan() || length_sqr <= max_distance {
                        Log::debug("actions", &format!("{:?} arrive at {:?}", obj_id, to));

                        let location = Location::Space {
                            sector_id: *sector_id,
                            pos: to.clone()
                        };

                        state.action = Action::Idle;
                        locations.set_location(obj_id, location);
                    } else {

                        let new_position = pos.add(&mov);
                        Log::debug("actions", &format!("{:?} move to {:?}", obj_id, new_position));

                        let location = Location::Space {
                            sector_id: *sector_id,
                            pos: new_position
                        };
                        locations.set_location(obj_id, location);
                    };
                },
                (Action::Jump, Location::Space { sector_id, pos}) => {
                    let jump = sectors.find_jump_at(sector_id, pos).unwrap();
                    Log::debug("actions", &format!("{:?} jump to {:?}", obj_id, jump.to));

                    let location = Location::Space {
                        sector_id: jump.to,
                        pos: jump.pos
                    };

                    state.action = Action::Idle;
                    locations.set_location(obj_id, location);
                },
                _ => {
                    Log::warn("actions", &format!("unknown {:?}", obj_id));
                }
            }
        }
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
