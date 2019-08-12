use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;

pub fn execute(actions: &mut Actions, locations: &mut Locations) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
            Action::Dock { target } => {
                let location = locations.get_location(obj_id).unwrap().get_space_opt();
                let target_location = locations.get_location(&target).unwrap().get_space_opt();

                match (&location, &target_location) {
                    (Some(loc), Some(tloc)) if loc.sector_id == tloc.sector_id &&
                        loc.pos.sub(&tloc.pos).length_sqr() <= MIN_DISTANCE_SQR => {

                        state.action = Action::Idle;
                        locations.set_location(obj_id, Location::Docked { obj_id: target });
                    },
                    _ => {
                        Log::warn("executor_action_dockundock", &format!("{:?} {:?} can not dock at {:?} {:?} has no location", obj_id, location, target, target_location));
                    }
                }

            },
            Action::Undock => {
                let location = locations.get_location(obj_id);

                match location {
                    Some(Location::Docked { obj_id: station_id }) => {
                        let station_location = locations.get_location(&station_id).unwrap();

                        let (sector_id, pos) = match station_location {
                            Location::Space { sector_id, pos } => {
                                (sector_id.clone(), pos.clone())
                            },
                            _ => {
                                Log::warn("executor_action_dockundock", &format!("{:?} can not undock, {:?} has no location", obj_id, station_id));
                                continue;
                            }
                        };

                        let new_location = Location::Space {
                            sector_id,
                            pos
                        };

                        state.action = Action::Idle;
                        locations.set_location(obj_id, new_location);
                    },
                    Some(Location::Space { .. }) => {
                        state.action = Action::Idle;
                    },
                    None => {
                        Log::warn("executor_action_dockundock", &format!("{:?} can not undock, has no location", obj_id));
                    }
                }
            },
            _ => {},
        }
    }
}
