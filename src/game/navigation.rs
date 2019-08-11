use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;

#[derive(Clone, Debug)]
struct State {
}


impl State {
    pub fn new() -> Self {
        State {
        }
    }
}

pub struct Navigations {
    index: HashMap<ObjId, State>,
}

impl Navigations {
    pub fn new() -> Self {
        Navigations {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId) {
        Log::info("navigations", &format!("init {:?}", id));
        self.index.insert(*id, State::new());
    }

//
//    pub fn set_location(&mut self, obj_id: &ObjId, value: Value) {
//        let mut state = self.index.get_mut(&obj_id).unwrap();
//        Log::info("template", &format!("set {:?}: {:?}", obj_id, value));
//        state.value = value;
//    }
//
//
//    pub fn get_location(&self, id: &ObjId) -> &Value {
//        let state = self.index.get(id).unwrap();
//        &state.value
//    }
}
