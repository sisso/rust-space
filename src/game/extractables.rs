use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;
use crate::game::wares::WareId;


#[derive(Clone,Debug)]
pub struct Extractable {
    pub ware_id: WareId,
    pub time: Seconds,
}


#[derive(Clone, Debug)]
struct State {
    extractable: Extractable
}

pub struct Extractables {
    index: HashMap<ObjId, State>,
}

impl Extractables {
    pub fn new() -> Self {
        Extractables {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId, extractable: Extractable) {
        self.set_extractable(id, extractable);
    }

    pub fn set_extractable(&mut self, obj_id: &ObjId, extractable: Extractable) {
        Log::info("extractable", &format!("set {:?}: {:?}", obj_id, extractable));

        if self.index.contains_key(obj_id) {
            let mut state = self.index.get_mut(&obj_id).unwrap();
            state.extractable = extractable;
        } else {
            let mut state = State { extractable };

            self.index.insert(*obj_id, state);
        }
    }

    pub fn get_extractable(&self, id: &ObjId) -> &Extractable {
        let state = self.index.get(id).unwrap();
        &state.extractable
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }
}
