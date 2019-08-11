use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;
use crate::game::sectors::{Sector, SectorRepo};

pub fn execute(actions: &mut Actions, locations: &mut Locations, sectors: &SectorRepo) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
            Action::Jump => {
                let location = locations.get_location(obj_id);

                match location {
                    Some(Location::Space { sector_id, pos}) => {
                        let jump = sectors.find_jump_at(sector_id, pos);

                        match jump {
                            Some(jump) => {
                                Log::debug("executor_action_jump", &format!("{:?} jump to {:?}", obj_id, jump.to));

                                let location = Location::Space {
                                    sector_id: jump.to,
                                    pos: jump.pos
                                };

                                state.action = Action::Idle;
                                locations.set_location(obj_id, location);
                            },
                            None => {
                                Log::warn("executor_action_jump", &format!("{:?} fail to jump, no jump at {:?} {:?}", obj_id, sector_id, pos));
                            }
                        }
                    },
                    _ => {
                        Log::warn("executor_action_jump", &format!("{:?} fail to jump, unknown location {:?}", obj_id, location));
                    }
                }
            },
            _ => {},
        }
    }
}
