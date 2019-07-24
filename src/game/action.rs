use crate::utils::{V2, Position, Log};
use crate::game::objects::{ObjRepo, Location, ObjId};
use crate::game::sectors::SectorRepo;
use crate::game::Tick;

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

pub struct Actions {

}

impl Actions {
    pub fn new() -> Self {
        Actions {}
    }

    pub fn tick(&mut self, tick: &Tick, objects: &mut ObjRepo, sectors: &SectorRepo) {
        let mut set_actions: Vec<(ObjId, Option<Action>, Option<Location>)> = vec![];

        for obj in objects.list() {
            match (&obj.action, &obj.location) {
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
                objects.set_action(obj_id, action);
            });

            if let Some(location) = location {
                objects.set_location(obj_id, location);
            }
        }
    }
}
