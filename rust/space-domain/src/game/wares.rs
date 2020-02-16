use specs::prelude::*;

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::utils::*;

use super::objects::ObjId;
use crate::game::jsons::JsonValueExtra;
use std::borrow::Borrow;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct WareId(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct WareAmount(pub WareId, pub f32);

#[derive(Debug, Clone, Component)]
pub struct Cargo {
    // TODO: u32
    max: f32,
    // TODO: u32
    current: f32,
    // TODO: back to hashset? since operations order can change depending of the order, should we keep tree?
    wares: BTreeMap<WareId, f32>,
}

#[derive(Debug,Clone)]
pub struct CargoTransfer {
    pub moved: Vec<(WareId, f32)>,
}

impl Cargo {
    pub fn new(max: f32) -> Self {
        Cargo {
            max,
            current: 0.0,
            wares: BTreeMap::new(),
        }
    }

    /// Currently impl leave a unknown state on failure, this is the reasons panik
    pub fn apply_move_from(&mut self, change: &CargoTransfer) -> Result<(), ()> {
        for (ware_id, amount) in change.moved.iter() {
            self.remove(*ware_id, *amount)
                .expect("apply move from failed");
        }

        Ok(())
    }

    /// Currently impl leave a unknown state on failure, this is the reasons panik
    pub fn apply_move_to(&mut self, change: &CargoTransfer) -> Result<(), ()> {
        for (ware_id, amount) in change.moved.iter() {
            self.add(*ware_id, *amount).expect("apply move to failed");
        }

        Ok(())
    }

    /// Move all cargo possible from to
    pub fn move_all_to_max(from: &Cargo, to: &Cargo) -> CargoTransfer {
        let mut change = CargoTransfer { moved: vec![] };

        let mut free_space = to.free_space();
        let mut total_moved = 0.0;

        for (id, amount) in from.wares.iter() {
            let available = free_space - total_moved;
            let amount_to_move = amount.min(available);

            if amount_to_move <= 0.0 {
                break;
            }

            change.moved.push((*id, amount_to_move));
            free_space -= amount_to_move;
        }

        return change;
    }

    pub fn remove(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        let ware_amount = *self.wares.get(&ware_id).unwrap_or(&0.0);
        if ware_amount < amount {
            return Result::Err(());
        }
        let new_amount = ware_amount - amount;
        if new_amount <= 0.0 {
            self.wares.remove(&ware_id);
        } else {
            self.wares.insert(ware_id, new_amount);
        }
        self.current -= amount;

        Result::Ok(())
    }

    pub fn add(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        if self.free_space() < amount {
            return Result::Err(());
        }

        let ware_amount = *self.wares.get(&ware_id).unwrap_or(&0.0);
        self.wares.insert(ware_id, ware_amount + amount);
        self.current += amount;

        Result::Ok(())
    }

    /// Add all cargo possible from to.
    pub fn add_to_max(&mut self, ware_id: WareId, amount: f32) -> f32 {
        let to_add = amount.min(self.free_space());
        self.add(ware_id, to_add).map(|i| to_add).unwrap_or(0.0)
    }

    /// Clear cargo only, leave configuration
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

    pub fn get_amount(&self, ware_id: WareId) -> f32 {
        self.wares.get(&ware_id).map(|i| *i).unwrap_or(0.0)
    }

    pub fn get_total(&self) -> f32 {
        self.max
    }
}

pub struct Cargos;

impl Cargos {
    pub fn new() -> Self {
        Cargos {}
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
    }

    pub fn move_all(cargos: &mut WriteStorage<Cargo>, from_id: ObjId, to_id: ObjId) -> CargoTransfer {
        let cargo_from = cargos.get(from_id).expect("Entity cargo not found");
        let cargo_to = cargos.get(to_id).expect("Deliver cargo not found");
        let transfer = Cargo::move_all_to_max(cargo_from, cargo_to);

        let cargo_from = cargos.get_mut(from_id).expect("Entity cargo not found");
        cargo_from.apply_move_from(&transfer).unwrap();

        let cargo_to = cargos.get_mut(to_id).expect("Deliver cargo not found");
        cargo_to.apply_move_to(&transfer).unwrap();

        transfer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const WARE0: WareId = WareId(0);
    const WARE1: WareId = WareId(1);
    const WARE2: WareId = WareId(2);

    #[test]
    fn test_cargo_transfer() {
        let mut cargo1 = Cargo::new(10.0);
        cargo1.add(WARE0, 4.0).unwrap();
        cargo1.add(WARE1, 3.0).unwrap();

        let mut cargo2 = Cargo::new(5.0);

        let change = Cargo::move_all_to_max(&cargo1, &cargo2);
        cargo1.apply_move_from(&change).unwrap();
        cargo2.apply_move_to(&change).unwrap();

        assert_eq!(0.0, cargo1.get_amount(WARE0));
        assert_eq!(2.0, cargo1.get_amount(WARE1));

        assert_eq!(4.0, cargo2.get_amount(WARE0));
        assert_eq!(1.0, cargo2.get_amount(WARE1));
    }

    #[test]
    fn test_cargo_add_over_capacity_should_fail() {
        let mut cargo = Cargo::new(1.0);
        let result = cargo.add(WARE0, 2.0);
        assert!(result.is_err())
    }

    #[test]
    fn test_cargo_add_to_max() {
        let mut cargo = Cargo::new(1.0);
        let amount = cargo.add_to_max(WARE0, 2.0);
        assert_eq!(1.0, amount);
        assert_eq!(1.0, cargo.get_current());

        let amount = cargo.add_to_max(WARE0, 2.0);
        assert_eq!(0.0, amount);
        assert_eq!(1.0, cargo.get_current());
    }
}
