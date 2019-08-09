use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct WareId(pub u32);

#[derive(Debug, Clone)]
pub struct Cargo {
    pub max: u32
}

impl Cargo {
    pub fn new(max: u32) -> Self {
        Cargo {
            max
        }
    }
}

#[derive(Clone, Debug)]
struct State {
    cargo: Cargo
}

impl State {
    pub fn new(max: u32) -> Self {
        State {
            cargo: Cargo::new(max)
        }
    }
}

pub struct Cargos {
    index: HashMap<ObjId, State>,
}

impl Cargos {
    pub fn new() -> Self {
        Cargos {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId, max: u32) {
        self.index.insert(*id, State::new(max));
    }

    pub fn get_cargo(&self, id: &ObjId) -> Option<&Cargo> {
        self.index.get(id).map(|i| &i.cargo)
    }
}

