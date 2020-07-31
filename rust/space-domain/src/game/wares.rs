use specs::prelude::*;

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

use super::objects::ObjId;
use crate::game::jsons::JsonValueExtra;
use std::borrow::Borrow;

pub type WareId = Entity;

#[derive(Debug, Clone, Component)]
pub struct Ware;

#[derive(Debug, Clone, Copy)]
pub struct WareAmount(pub WareId, pub f32);

impl WareAmount {
    pub fn get_ware_id(&self) -> WareId {
        self.0
    }

    pub fn get_amount(&self) -> f32 {
        self.1
    }
}

impl From<(WareId, f32)> for WareAmount {
    fn from((ware_id, amount): (WareId, f32)) -> Self {
        WareAmount(ware_id, amount)
    }
}

#[derive(Debug, Clone, Component)]
pub struct Cargo {
    max: f32,
    current: f32,
    wares: Vec<WareAmount>,
    /// When a whitelist is defined, the total cargo is equally distributed between the wares.
    /// Any other ware is not accepted
    whitelist: Vec<WareId>,
}

impl Cargo {
    pub fn new(max: f32) -> Self {
        Cargo {
            max,
            current: 0.0,
            wares: Default::default(),
            whitelist: Default::default(),
        }
    }

    pub fn set_whitelist(&mut self, wares: Vec<WareId>) {
        self.whitelist = wares;
    }

    pub fn remove(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        match self.wares.iter().position(|i| i.0 == ware_id) {
            Some(pos) if self.wares[pos].1 >= amount => {
                self.wares[pos].1 -= amount;
                self.current -= amount;
                Result::Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn add(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        if self.free_space(ware_id) < amount {
            return Result::Err(());
        }

        match self.wares.iter().position(|i| i.0 == ware_id) {
            Some(pos) => {
                self.wares[pos].1 += amount;
            }
            None => {
                self.wares.push(WareAmount(ware_id, amount));
            }
        }

        self.current += amount;
        Result::Ok(())
    }

    /// add all wares or none
    pub fn add_all(&mut self, wares: &Vec<WareAmount>) -> Result<(), ()> {
        for WareAmount(ware_id, amount) in wares {
            if self.free_space(*ware_id) < *amount {
                return Err(());
            }
        }

        for WareAmount(ware_id, amount) in wares {
            self.add(*ware_id, *amount).unwrap();
        }

        Ok(())
    }

    /// remove all wares or none
    pub fn remove_all(&mut self, wares: &Vec<WareAmount>) -> Result<(), ()> {
        for WareAmount(ware_id, amount) in wares {
            if self.get_amount(*ware_id) < *amount {
                return Err(());
            }
        }

        for WareAmount(ware_id, amount) in wares {
            self.remove(*ware_id, *amount).unwrap();
        }

        Ok(())
    }

    /// Add all cargo possible from to.
    pub fn add_to_max(&mut self, ware_id: WareId, amount: f32) -> f32 {
        let to_add = amount.min(self.free_space(ware_id));

        self.add(ware_id, to_add).map(|i| to_add).unwrap_or(0.0)
    }

    /// Clear cargo only, leave configuration
    pub fn clear(&mut self) {
        self.current = 0.0;
        self.wares.clear();
    }

    pub fn free_space(&self, ware_id: WareId) -> f32 {
        if self.whitelist.is_empty() {
            self.max - self.current
        } else {
            if self.whitelist.iter().find(|i| **i == ware_id).is_none() {
                return 0.0;
            }

            let share = self.max / self.whitelist.len() as f32;
            share - self.get_amount(ware_id)
        }
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn is_empty(&self) -> bool {
        self.current <= 0.001
    }

    pub fn get_current(&self) -> f32 {
        self.current
    }

    pub fn get_wares<'a>(&'a self) -> impl Iterator<Item = WareId> + 'a {
        self.wares
            .iter()
            .filter(|WareAmount(_, amount)| *amount > 0.0)
            .map(|WareAmount(ware_id, _)| *ware_id)
    }

    pub fn get_amount(&self, ware_id: WareId) -> f32 {
        self.wares
            .iter()
            .find_map(|WareAmount(i_ware_id, i_amount)| {
                if ware_id == *i_ware_id {
                    Some(*i_amount)
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    pub fn get_max(&self) -> f32 {
        self.max
    }
}

#[derive(Debug, Clone)]
pub struct CargoTransfer {
    pub moved: Vec<WareAmount>,
}

impl CargoTransfer {
    /// Move all cargo possible from to
    pub fn transfer_all(from: &Cargo, to: &Cargo) -> CargoTransfer {
        CargoTransfer::transfer_impl(from, to, None)
    }

    pub fn transfer_only(from: &Cargo, to: &Cargo, wares: &Vec<WareId>) -> CargoTransfer {
        CargoTransfer::transfer_impl(from, to, Some(wares))
    }

    fn transfer_impl(from: &Cargo, to: &Cargo, wares: Option<&Vec<WareId>>) -> CargoTransfer {
        let mut change = CargoTransfer { moved: vec![] };

        // use a temporary copy to simulate the transfer
        let mut tmp_to = to.clone();

        for WareAmount(id, amount) in &from.wares {
            if let Some(wares) = wares {
                if !wares.contains(id) {
                    continue;
                }
            }

            let available = tmp_to.free_space(*id);
            let amount_to_move = amount.min(available);
            if amount_to_move > 0.0 {
                tmp_to.add(*id, amount_to_move).unwrap();
                change.moved.push(WareAmount(*id, amount_to_move));
            }
        }

        return change;
    }

    fn apply_move_from(&self, cargo: &mut Cargo) -> Result<(), ()> {
        cargo.remove_all(&self.moved)
    }

    fn apply_move_to(&self, cargo: &mut Cargo) -> Result<(), ()> {
        cargo.add_all(&self.moved)
    }
}

pub struct Cargos;

impl Cargos {
    pub fn move_only(
        cargos: &mut WriteStorage<Cargo>,
        from_id: ObjId,
        to_id: ObjId,
        wares: &Vec<WareId>,
    ) -> CargoTransfer {
        Cargos::move_impl(cargos, from_id, to_id, Some(wares))
    }

    pub fn move_all(
        cargos: &mut WriteStorage<Cargo>,
        from_id: ObjId,
        to_id: ObjId,
    ) -> CargoTransfer {
        Cargos::move_impl(cargos, from_id, to_id, None)
    }

    fn move_impl(
        cargos: &mut WriteStorage<Cargo>,
        from_id: ObjId,
        to_id: ObjId,
        wares: Option<&Vec<WareId>>,
    ) -> CargoTransfer {
        let cargo_from = cargos.get(from_id).expect("Entity cargo not found");
        let cargo_to = cargos.get(to_id).expect("Deliver cargo not found");
        let transfer = CargoTransfer::transfer_impl(cargo_from, cargo_to, wares);

        trace!(
            "move wares {:?} from {:?} to {:?}, transfer is {:?}",
            wares,
            cargo_from,
            cargo_to,
            transfer
        );

        let cargo_from = cargos.get_mut(from_id).expect("Entity cargo not found");
        transfer
            .apply_move_from(cargo_from)
            .expect("To remove wares to be transfer");

        let cargo_to = cargos.get_mut(to_id).expect("Deliver cargo not found");
        transfer
            .apply_move_to(cargo_to)
            .expect("To add wares to be transfer");

        transfer
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use specs::world::Generation;

    // TODO: how to create entities without a world?
    fn create_wares() -> (WareId, WareId, WareId) {
        let mut world = World::new();
        let ware_0 = world.create_entity().build();
        let ware_1 = world.create_entity().build();
        let ware_2 = world.create_entity().build();
        (ware_0, ware_1, ware_2)
    }

    #[test]
    fn test_cargo_transfer() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo_from = Cargo::new(10.0);
        cargo_from.add(ware_0, 4.0).unwrap();
        cargo_from.add(ware_1, 3.0).unwrap();

        let mut cargo_to = Cargo::new(5.0);

        let transfer = CargoTransfer::transfer_all(&cargo_from, &cargo_to);
        transfer.apply_move_from(&mut cargo_from).unwrap();
        transfer.apply_move_to(&mut cargo_to).unwrap();

        assert_eq!(0.0, cargo_from.get_amount(ware_0));
        assert_eq!(2.0, cargo_from.get_amount(ware_1));

        assert_eq!(4.0, cargo_to.get_amount(ware_0));
        assert_eq!(1.0, cargo_to.get_amount(ware_1));
    }

    #[test]
    fn test_cargo_transfer_only() {
        let (ware_0, ware_1, ware_2) = create_wares();

        let mut cargo_from = Cargo::new(10.0);
        cargo_from.add(ware_0, 2.0).unwrap();
        cargo_from.add(ware_1, 2.0).unwrap();
        cargo_from.add(ware_2, 2.0).unwrap();

        let mut cargo_to = Cargo::new(5.0);

        let transfer = CargoTransfer::transfer_only(&cargo_from, &cargo_to, &vec![ware_0, ware_1]);
        transfer.apply_move_from(&mut cargo_from).unwrap();
        transfer.apply_move_to(&mut cargo_to).unwrap();

        assert_eq!(0.0, cargo_from.get_amount(ware_0));
        assert_eq!(0.0, cargo_from.get_amount(ware_1));
        assert_eq!(2.0, cargo_from.get_amount(ware_2));

        assert_eq!(2.0, cargo_to.get_amount(ware_0));
        assert_eq!(2.0, cargo_to.get_amount(ware_1));
        assert_eq!(0.0, cargo_to.get_amount(ware_2));
    }

    #[test]
    fn test_cargo_add_over_capacity_should_fail() {
        let (ware_0, _, _ware_2) = create_wares();
        let mut cargo = Cargo::new(1.0);
        let result = cargo.add(ware_0, 2.0);
        assert!(result.is_err())
    }

    #[test]
    fn test_cargo_add_to_max() {
        let (ware_0, _, _ware_2) = create_wares();

        let mut cargo = Cargo::new(1.0);
        let amount = cargo.add_to_max(ware_0, 2.0);
        assert_eq!(1.0, amount);
        assert_eq!(1.0, cargo.get_current());

        let amount = cargo.add_to_max(ware_0, 2.0);
        assert_eq!(0.0, amount);
        assert_eq!(1.0, cargo.get_current());
    }

    #[test]
    fn test_cargo_whitelist_should_reject_any_other_ware() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0]);

        // with invalid ware
        assert!(cargo.add(ware_1, 1.0).is_err());
        assert_eq!(cargo.add_to_max(ware_1, 1.0), 0.0);
        assert!(cargo.add_all(&vec![WareAmount(ware_1, 1.0)]).is_err());
        assert_eq!(cargo.free_space(ware_1), 0.0);
    }

    #[test]
    fn test_cargo_whitelist_should_accept_valid_ware() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0]);

        assert_eq!(cargo.free_space(ware_0), 10.0);
        assert!(cargo.add(ware_0, 2.0).is_ok());
        assert!(cargo.add_all(&vec![WareAmount(ware_0, 2.0)]).is_ok());
        assert_eq!(cargo.get_amount(ware_0), 4.0);
        assert_eq!(cargo.add_to_max(ware_0, 20.0), 6.0);
    }

    #[test]
    fn test_cargo_whitelist_should_split_cargo_even() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0, ware_1]);

        for ware_id in vec![ware_0, ware_1] {
            assert_eq!(cargo.free_space(ware_id), 5.0);
            assert!(cargo.add(ware_id, 2.0).is_ok());
            assert!(cargo.add_all(&vec![WareAmount(ware_id, 2.0)]).is_ok());
            assert_eq!(cargo.get_amount(ware_id), 4.0);
            assert_eq!(cargo.free_space(ware_id), 1.0);
            assert_eq!(cargo.add_to_max(ware_id, 20.0), 1.0);
            assert_eq!(cargo.get_amount(ware_id), 5.0);
            assert_eq!(cargo.free_space(ware_id), 0.0);
            assert!(cargo.add(ware_id, 1.0).is_err());
        }
    }

    #[test]
    fn test_cargo_should_not_return_empty_lists() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.add(ware_0, 4.0).unwrap();
        cargo.remove(ware_0, 4.0).unwrap();

        assert!(cargo.get_wares().next().is_none());
    }
}
