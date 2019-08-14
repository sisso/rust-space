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

    /** Move all cargo possible from to  */
    pub fn move_all_to_max(from: &mut Cargo, to: &mut Cargo) {
        for (id, amount) in from.wares.clone() {
            let available = to.free_space();
            let amount_to_move = amount.min(available);

            if let Result::Err(()) = to.add(&id, amount_to_move) {
                return;
            }

            from.remove(&id, amount_to_move);
        }
    }

    pub fn remove(&mut self, ware_id: &WareId, amount: f32) -> Result<(),()> {
        let ware_amount = *self.wares.get(ware_id).unwrap_or(&0.0);
        if ware_amount < amount {
            return Result::Err(());
        }
        let new_amount = ware_amount - amount;
        if new_amount <= 0.0 {
            self.wares.remove(ware_id);
        } else {
            self.wares.insert(*ware_id, new_amount);
        }
        self.current -= amount;

        Result::Ok(())
    }

    pub fn add(&mut self, ware_id: &WareId, amount: f32) -> Result<(),()> {
        if self.free_space() < amount {
            return Result::Err(());
        }

        let ware_amount = *self.wares.get(ware_id).unwrap_or(&0.0);
        self.wares.insert(*ware_id, ware_amount + amount);
        self.current += amount;

        Result::Ok(())
    }

    /** Add all cargo possible from to */
    pub fn add_to_max(&mut self, ware_id: &WareId, amount: f32) {
        let to_add  = amount.min(self.free_space());
        let _ = self.add(ware_id, to_add);
    }

    pub fn clear(&mut self) {
        self.current = 0.0;
        self.wares.clear();
    }

    pub fn free_space(&self) -> f32 {
        self.max - self.current
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn get_current(&self) -> f32 {
        self.current
    }

    pub fn get_wares(&self) -> Vec<&WareId> {
        self.wares.keys().collect()
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

    pub fn move_all(&mut self, from: &ObjId, to: &ObjId) {
        let mut cargo_to= self.index.remove(to).unwrap();
        let cargo_from = self.index.get_mut(from).unwrap();
        Cargo::move_all_to_max(&mut cargo_from.cargo, &mut cargo_to.cargo);
        self.index.insert(*to, cargo_to);
        Log::info("Cargos", &format!("move_all {:?} to {:?}, new cargos {:?} and {:?}", from, to, self.index.get(from), self.index.get(to)));
    }
}

