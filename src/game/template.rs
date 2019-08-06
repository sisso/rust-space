use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;

#[derive(Clone, Debug)]
struct State {
    value: Value
}

#[derive(Clone, Debug)]
pub struct Value {
}

impl State {
    pub fn new() -> Self {
        State {
            value: Value {}
        }
    }
}

pub struct Templates {
    index: HashMap<ObjId, State>,
}

impl Templates {
    pub fn new() -> Self {
        Templates {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId) {
        self.index.insert(*id, State::new());
    }

    pub fn set_location(&mut self, obj_id: &ObjId, value: Value) {
        let mut state = self.index.get_mut(&obj_id).unwrap();
        Log::info("template", &format!("set {:?}: {:?}", obj_id, value));
        state.value = value;
    }


    pub fn get_location(&self, id: &ObjId) -> &Value {
        let state = self.index.get(id).unwrap();
        &state.value
    }
}
