use crate::game::actions::*;
use crate::game::objects::{ObjId, Obj};
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;
use crate::game::sectors::{Sector, Sectors};

pub fn execute(actions: &mut Actions, locations: &mut Locations, sectors: &Sectors) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
            Action::Jump => {
                let location = locations.get_location(obj_id);

                match location {
                    Some(Location::Space { sector_id, pos}) => {
                        let jump = match sectors.find_jump_at(sector_id, pos) {
                            Some(jump) => jump,
                            None => {
                                warn!("executor_action_jump", &format!("{:?} fail to jump, no jump at {:?} {:?}", obj_id, sector_id, pos));
                                continue;
                            }
                        };

                        let target_jump = sectors.find_target_jump(jump.to, *sector_id);

                        debug!("executor_action_jump", &format!("{:?} jump from {:?} at {:?} to {:?} at {:?}", obj_id, sector_id, pos, jump.to, target_jump.pos));

                        let location = Location::Space {
                            sector_id: jump.to,
                            pos: target_jump.pos
                        };

                        state.action = Action::Idle;
                        locations.set_location(obj_id, location);
                    },
                    _ => {
                        warn!("executor_action_jump", &format!("{:?} fail to jump, unknown location {:?}", obj_id, location));
                    }
                }
            },
            _ => {},
        }
    }
}
