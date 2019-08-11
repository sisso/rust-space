use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;

pub fn execute(actions: &mut Actions, locations: &mut Locations) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
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
