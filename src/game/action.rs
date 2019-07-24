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
                    let length_sqr = delta.length_sqr();

                    if length_sqr <= MIN_DISTANCE_SQR {
                        Log::debug("actions", &format!("{:?} arrive at {:?}", obj.id, to));
                        set_actions.push((obj.id, Some(Action::Idle), None));
                    } else {
                        let delta = delta.normalized();
                        let delta = delta.mult(tick.delta_time.0);
                        let new_position = pos.add(&delta);
                        Log::debug("actions", &format!("{:?} move to {:?}", obj.id, new_position));
                        set_actions.push((obj.id, None, Some(Location::Space {
                            sector_id: *sector_id,
                            pos: new_position
                        })));
                    }
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
