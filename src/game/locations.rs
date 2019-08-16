use super::objects::*;
use super::sectors::*;
use crate::utils::*;

use std::collections::HashMap;
use crate::game::save::Save;

#[derive(Clone, Debug)]
pub enum Location {
    Docked { docked_id: ObjId },
    Space { sector_id: SectorId, pos: Position },
}

#[derive(Clone, Debug)]
pub struct LocationSpace {
    pub sector_id: SectorId,
    pub pos: Position
}

#[derive(Clone, Debug)]
pub struct Moveable {
    pub speed: Speed
}

impl Location {
    pub fn get_space(&self) -> LocationSpace {
        match self {
            Location::Space { sector_id, pos} => LocationSpace { sector_id: *sector_id, pos: *pos },
            _ => panic!("unexpected state for get")
        }
    }

    pub fn get_space_opt(&self) -> Option<LocationSpace> {
        match self {
            Location::Space { sector_id, pos} => Some(LocationSpace { sector_id: *sector_id, pos: *pos }),
            _ => None
        }
    }

    pub fn get_docked(&self) -> ObjId {
        match self {
            Location::Docked { docked_id } => *docked_id,
            _ => panic!("unexpected state for get_docked")
        }
    }
}

struct State {
    location: Option<Location>,
    movement: Option<Moveable>,
}

impl State {
    pub fn new() -> Self {
        State {
            location: None,
            movement: None,
        }
    }
}

pub struct Locations {
    index: HashMap<ObjId, State>,
}

impl Locations {
    pub fn new() -> Self {
        Locations {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId) {
        self.index.insert(*id, State::new());
    }

    pub fn set_location(&mut self, obj_id: &ObjId, location: Location) {
        let state = self.get_create_state(&obj_id);
        Log::info("locations", &format!("set location {:?}: {:?}", obj_id, location));
        state.location = Some(location);
    }

    pub fn set_moveable(&mut self, obj_id: &ObjId, speed: Speed) {
        let state = self.get_create_state(&obj_id);
        Log::info("locations", &format!("set moveable {:?}: {:?}", obj_id, speed));
        state.movement = Some(Moveable { speed, });
    }

    pub fn get_location(&self, id: &ObjId) -> Option<&Location> {
        let state = self.index.get(id);
        state.and_then(|i| i.location.as_ref())
    }

    pub fn get_speed(&self, id: &ObjId) -> Option<&Speed> {
        let state = self.index.get(id);
        state.and_then(|i| {
            i.movement.as_ref().map(|j| &j.speed)
        })
    }

    fn get_create_state(&mut self, obj_id: &&ObjId) -> &mut State {
        let mut state = self.index.get_mut(&obj_id).unwrap();
        state
    }


    pub fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (k,v) in self.index.iter() {
            let speed: Option<f32> = match v.movement {
                Some(Moveable{ speed }) => Some(speed.0),
                None => None
            };

            let (sector_id, docket_at, pos) = match v.location {
                Some(Location::Space { sector_id, pos })=> {
                    (Some(sector_id.0), None, Some((pos.x, pos.y)))
                }
                Some(Location::Docked { docked_id }) => {
                    (None, Some(docked_id.0), None)
                }
                None => {
                    (None, None, None)
                }
            };

            save.add(k.0, "location", json!({
                "sector_id": sector_id,
                "docket_at": docket_at,
                "pos": pos,
                "speed": speed,
            }));
        }
    }
}
