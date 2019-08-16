use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;
use crate::game::sectors::{Sector, Sectors, SectorId};
use crate::game::Tick;

pub fn execute(tick: &Tick, actions: &mut Actions, locations: &mut Locations, sectors: &Sectors) {
    for (obj_id, state) in actions.list_mut() {
        match state.action {
            Action::Fly { to } => {
                let location = locations.get_location(obj_id);

                match location {
                    Some(Location::Space { sector_id, pos }) => {
                        execute_fly(tick, locations, sectors, obj_id, state, *pos, *sector_id, to);
                    },
                    _ => {
                        Log::debug("executor_actions_fly", &format!("{:?} can not fly, invalid {:?} location", obj_id, location));
                    }
                }

            },
            _ => {},
        }
    }
}

fn execute_fly(tick: &Tick,
               locations: &mut Locations,
               sectors: &Sectors,
               obj_id: &ObjId,
               state: &mut ActionState,
               pos: Position,
               sector_id: SectorId,
               to: Position) {

    let delta   = to.sub(&pos);
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
            sector_id: sector_id,
            pos: to.clone()
        };

        state.action = Action::Idle;
        locations.set_location(obj_id, location);
    } else {

        let new_position = pos.add(&mov);
        Log::debug("actions", &format!("{:?} move to {:?}", obj_id, new_position));

        let location = Location::Space {
            sector_id: sector_id,
            pos: new_position
        };
        locations.set_location(obj_id, location);
    };
}
