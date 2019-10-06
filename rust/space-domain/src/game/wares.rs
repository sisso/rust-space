use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage};

use std::collections::{HashMap, BTreeMap, HashSet};
use serde_json::{json, Value};

use crate::game::save::{CanSave, CanLoad, Load, Save};
use crate::utils::*;

use super::objects::ObjId;
use crate::game::jsons::JsonValueExtra;
use std::borrow::Borrow;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug,PartialOrd,Ord)]
pub struct WareId(pub u32);

#[derive(Debug,Clone,Copy)]
pub struct WareAmount(pub WareId, pub f32);

#[derive(Debug, Clone)]
pub struct Cargo {
    max: f32,
    current: f32,
    // TODO: back to hashset? since operations order can change depending of the order, should we keep tree?
    wares: BTreeMap<WareId, f32>,
}

impl Cargo {
    pub fn new(max: f32) -> Self {
        Cargo {
            max,
            current: 0.0,
            wares: BTreeMap::new(),
        }
    }

    /// Move all cargo possible from to
    pub fn move_all_to_max(from: &mut Cargo, to: &mut Cargo) {
        for (id, amount) in from.wares.clone() {
            let available = to.free_space();
            let amount_to_move = amount.min(available);

            if amount_to_move <= 0.0 {
                return;
            }

            if to.add(id, amount_to_move).is_err() {
                return;
            }

            assert!(from.remove(id, amount_to_move).is_ok());
        }
    }

    pub fn remove(&mut self, ware_id: WareId, amount: f32) -> Result<(),()> {
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

    pub fn add(&mut self, ware_id: WareId, amount: f32) -> Result<(),()> {
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
        let to_add  = amount.min(self.free_space());
        self.add(ware_id, to_add)
            .map(|i| to_add)
            .unwrap_or(0.0)
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
}

impl SpecComponent for Cargo {
    type Storage = VecStorage<Self>;
}

pub fn init_world(world: &mut World) {
    world.register::<Cargo>();
}

pub struct Cargos;

impl Cargos {
    pub fn new() -> Self {
        Cargos {
        }
    }
    pub fn move_all(from: &ObjId, to: &ObjId) {
        unimplemented!();

//        let mut cargo_to= self.index.remove(to).unwrap();
//        let cargo_from = self.index.get_mut(from).unwrap();
//        Cargo::move_all_to_max(&mut cargo_from.cargo, &mut cargo_to.cargo);
//        self.index.insert(*to, cargo_to);
//        info!("Cargos", &format!("move_all {:?} to {:?}, new cargos {:?} and {:?}", from, to, self.index.get(from), self.index.get(to)));
    }
}

impl CanSave for Cargos {
    fn save(&self, save: &mut impl Save) {
//        for (obj_id,state) in self.index.iter() {
//            let wares_json: Vec<Value> =
//                state.cargo.wares.iter().map(|(ware_id, amount)| {
//                    json!({
//                        "ware_id": ware_id.0,
//                        "amount": *amount,
//                    })
//                }).collect();
//
//            save.add(obj_id.0, "cargo", json!({
//                "max": state.cargo.max,
//                "current": state.cargo.current,
//                "wares": wares_json
//            }));
//        }
    }
}

impl CanLoad for Cargos {
    fn load(&mut self, load: &mut impl Load) {
//        for (k, v) in load.get_components("cargo") {
//            let wares: BTreeMap<WareId, f32> =
//                v["wares"].as_array().unwrap().iter().map(|i| {
//                    let ware_id = WareId(i["ware_id"].to_u32());
//                    let amount = i["amount"].to_f32();
//                    (ware_id, amount)
//                }).collect();
//
//            self.index.insert(ObjId(*k), State {
//                cargo: Cargo {
//                    max: v["max"].to_f32(),
//                    current: v["current"].to_f32(),
//                    wares,
//                }
//            });
//        }
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
        cargo1.add(WARE0, 4.0);
        cargo1.add(WARE1, 3.0);

        let mut cargo2 = Cargo::new(5.0);

        Cargo::move_all_to_max(&mut cargo1, &mut cargo2);
        println!("after move_all_to_max:\n{:?}\n{:?}", cargo1, cargo2);

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
