use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct WareId(pub u32);

#[derive(Debug, Clone)]
pub struct Cargo {
    pub max: f32,
    current: f32,
    wares: HashMap<WareId, f32>,
}

impl Cargo {
    pub fn new(max: f32) -> Self {
        Cargo {
            max,
            current: 0.0,
            wares: HashMap::new(),
        }
    }

    pub fn add_to_max(&mut self, ware_id: &WareId, amount: f32) {
        let max = self.free_space();
        let to_add  = amount.min(max);

        if to_add <= 0.0 {
            return;
        }

        let e = self.wares.get(ware_id);
        let c  = e.unwrap_or(&0.0);

        self.wares.insert(*ware_id, c + to_add);
        self.current += to_add;
    }

    pub fn free_space(&self) -> f32 {
        self.max - self.current
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
}

#[derive(Clone, Debug)]
struct State {
    cargo: Cargo
}

impl State {
    pub fn new(max: f32) -> Self {
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

    pub fn init(&mut self, id: &ObjId, max: f32) {
        self.index.insert(*id, State::new(max));
    }

    pub fn get_cargo(&self, id: &ObjId) -> Option<&Cargo> {
        self.index.get(id).map(|i| &i.cargo)
    }

    pub fn get_cargo_mut(&mut self, id: &ObjId) -> Option<&mut Cargo> {
        self.index.get_mut(id).map(|i| &mut i.cargo)
    }
}

