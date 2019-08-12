use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;

struct State {

}

impl State {
    pub fn new() -> Self {
        State {
        }
    }
}

pub struct Docking {
    index: HashMap<ObjId, State>,
}

impl Docking {
    pub fn new() -> Self {
        Docking {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, obj_id: &ObjId) {
        Log::info("docking", &format!("init {:?}", obj_id));
        self.index.insert(*obj_id, State::new());
    }

    pub fn set(&mut self, obj_id: &ObjId) {
        let mut state = self.index.get_mut(&obj_id).unwrap();
        Log::info("docking", &format!("set {:?}", obj_id));
    }


//    pub fn get(&self, id: &ObjId) -> &Value {
//        let state = self.index.get(id).unwrap();
//        &state.value
//    }
}
