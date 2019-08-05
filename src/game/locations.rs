use super::objects::*;
use super::sectors::*;
use crate::utils::*;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Location {
    Docked { obj_id: ObjId },
    Space { sector_id: SectorId, pos: Position },
}

#[derive(Clone, Debug)]
pub struct LocationSpace {
    pub sector_id: SectorId,
    pub pos: Position
}

impl Location {
    pub fn as_space(&self) -> LocationSpace {
        match self {
            Location::Space { sector_id, pos} => LocationSpace { sector_id: *sector_id, pos: *pos },
            _ => panic!("unexpected state for get")
        }
    }

    pub fn get(&self) -> (SectorId, Position) {
        match self {
            Location::Space { sector_id, pos} => (*sector_id, *pos),
            _ => panic!("unexpected state for get")
        }
    }

    pub fn get_docked(&self) -> ObjId {
        match self {
            Location::Docked { obj_id } => *obj_id,
            _ => panic!("unexpected state for get_docked")
        }
    }
}

struct State {
    location: Option<Location>
}

impl State {
    pub fn new() -> Self {
        State {
            location: None
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
        let mut state = self.index.get_mut(&obj_id).unwrap();
        Log::info("locations", &format!("set location {:?}: {:?}", obj_id, location));
        state.location = Some(location);
    }


    pub fn get_location(&self, id: &ObjId) -> Option<&Location> {
        let state = self.index.get(id);
        state.and_then(|i| i.location.as_ref())
    }
}
