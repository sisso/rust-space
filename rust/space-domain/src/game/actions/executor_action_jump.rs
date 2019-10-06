use crate::game::actions::*;
use crate::game::objects::{ObjId};
use crate::utils::*;
use crate::game::locations::*;
use crate::game::sectors::{Sector, Sectors};
use crate::game::events::{Events, ObjEvent, EventKind};

pub fn execute(actions: &mut Actions, locations: &mut Locations, sectors: &Sectors, events: &mut Events) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
            Action::Jump { jump_id } => {
                let location = locations.get_location(obj_id);

                match location {
                    Some(Location::Space { sector_id, pos}) => {
                        let jump = sectors.get_jump(jump_id).unwrap();

                        debug!("executor_action_jump", &format!("{:?} jump from {:?} at {:?} to {:?} at {:?}", obj_id, sector_id, pos, jump.to_sector_id, jump.to_pos));

                        let location = Location::Space {
                            sector_id: jump.to_sector_id,
                            pos: jump.to_pos
                        };

                        state.action = Action::Idle;
                        locations.set_location(obj_id, location);

                        events.add_obj_event(ObjEvent::new(*obj_id, EventKind::Jump));
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
