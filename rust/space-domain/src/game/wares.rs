use specs::prelude::*;

use super::objects::ObjId;

pub type WareId = Entity;

#[derive(Debug, Clone, Component)]
pub struct Ware;

#[derive(Debug, Clone, Copy)]
pub struct WareAmount {
    pub ware_id: WareId,
    pub amount: f32,
}

impl WareAmount {
    pub fn new(ware_id: WareId, amount: f32) -> Self {
        Self { ware_id, amount }
    }

    pub fn get_ware_id(&self) -> WareId {
        self.ware_id
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }
}

impl From<(WareId, f32)> for WareAmount {
    fn from((ware_id, amount): (WareId, f32)) -> Self {
        WareAmount::new(ware_id, amount)
    }
}

#[derive(Debug, Clone, Component)]
pub struct Cargo {
    max_volume: f32,
    current_volume: f32,
    wares: Vec<WareAmount>,
    /// When a whitelist is defined, the total cargo is equally distributed between the wares.
    /// Any other ware is not accepted
    whitelist: Vec<WareId>,
}

impl Cargo {
    pub fn new(max: f32) -> Self {
        Cargo {
            max_volume: max,
            current_volume: 0.0,
            wares: vec![],
            whitelist: vec![],
        }
    }

    pub fn get_wares(&self) -> &Vec<WareAmount> {
        &self.wares
    }

    pub fn set_whitelist(&mut self, wares: Vec<WareId>) {
        self.whitelist = wares;
    }

    pub fn remove(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        if let Some(index) = self.wares.iter().position(|i| i.ware_id == ware_id) {
            self.wares[index].amount -= amount;
            if is_near_zero(self.wares[index].amount) {
                self.wares.remove(index);
            } else {
                self.current_volume -= amount;
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add(&mut self, ware_id: WareId, amount: f32) -> Result<(), ()> {
        if self.free_volume(ware_id) < amount {
            return Err(());
        }

        match self.wares.iter().position(|i| i.ware_id == ware_id) {
            Some(pos) => {
                self.wares[pos].amount += amount;
            }
            None => {
                self.wares.push(WareAmount::new(ware_id, amount));
            }
        }

        self.current_volume += amount;
        Ok(())
    }

    /// add all wares or none
    pub fn add_all(&mut self, wares: &Vec<WareAmount>) -> Result<(), ()> {
        for w in wares {
            if self.free_volume(w.ware_id) < w.amount {
                return Err(());
            }
        }

        for w in wares {
            self.add(w.ware_id, w.amount).unwrap();
        }

        Ok(())
    }

    /// remove all wares or none
    pub fn remove_all(&mut self, wares: &Vec<WareAmount>) -> Result<(), ()> {
        for w in wares {
            if self.get_amount(w.ware_id) < w.amount {
                return Err(());
            }
        }

        for w in wares {
            self.remove(w.ware_id, w.amount).unwrap();
        }

        Ok(())
    }

    /// Add all cargo possible from to.
    pub fn add_to_max(&mut self, ware_id: WareId, amount: f32) -> f32 {
        let to_add = amount.min(self.free_volume(ware_id));

        self.add(ware_id, to_add).map(|_i| to_add).unwrap_or(0.0)
    }

    /// Clear cargo only, leave configuration
    pub fn clear(&mut self) {
        self.current_volume = 0.0;
        self.wares.clear();
    }

    pub fn free_volume(&self, ware_id: WareId) -> f32 {
        if self.whitelist.is_empty() {
            self.max_volume - self.current_volume
        } else {
            if !self.whitelist.contains(&ware_id) {
                return 0.0;
            }

            let share = self.max_volume / self.whitelist.len() as f32;
            share - self.get_amount(ware_id)
        }
    }

    pub fn is_full(&self) -> bool {
        self.current_volume >= self.max_volume
    }

    pub fn is_empty(&self) -> bool {
        self.current_volume <= 0.001
    }

    pub fn get_current_volume(&self) -> f32 {
        self.current_volume
    }

    pub fn get_wares_ids<'a>(&'a self) -> impl Iterator<Item = WareId> + 'a {
        self.wares
            .iter()
            .filter(|i| i.amount > 0.0)
            .map(|i| i.ware_id)
    }

    pub fn get_amount(&self, ware_id: WareId) -> f32 {
        self.wares
            .iter()
            .find(|i| i.ware_id == ware_id)
            .map(|i| i.amount)
            .unwrap_or(0.0)
    }

    pub fn get_max(&self) -> f32 {
        self.max_volume
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

        for w in &from.wares {
            if let Some(wares) = wares {
                if !wares.contains(&w.ware_id) {
                    continue;
                }
            }

            let available = tmp_to.free_volume(w.ware_id);
            let amount_to_move = w.amount.min(available);
            if amount_to_move > 0.0 {
                tmp_to.add(w.ware_id, amount_to_move).unwrap();
                change
                    .moved
                    .push(WareAmount::new(w.ware_id, amount_to_move));
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

        log::trace!(
            "move wares {:?} from {:?} to {:?}, transfer is {:?}",
            wares,
            cargo_from,
            cargo_to,
            transfer,
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

/// consider any value small that 0.01 as zero
/// TODO: should be better cargo be 1 * 100???"
pub fn is_near_zero(v: f32) -> bool {
    v < 0.01 && v > -0.01
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(1.0, cargo.get_current_volume());

        let amount = cargo.add_to_max(ware_0, 2.0);
        assert_eq!(0.0, amount);
        assert_eq!(1.0, cargo.get_current_volume());
    }

    #[test]
    fn test_cargo_whitelist_should_reject_any_other_ware() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0]);

        // with invalid ware
        assert!(cargo.add(ware_1, 1.0).is_err());
        assert_eq!(cargo.add_to_max(ware_1, 1.0), 0.0);
        assert!(cargo.add_all(&vec![WareAmount::new(ware_1, 1.0)]).is_err());
        assert_eq!(cargo.free_volume(ware_1), 0.0);
    }

    #[test]
    fn test_cargo_whitelist_should_accept_valid_ware() {
        let (ware_0, _ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0]);

        assert_eq!(cargo.free_volume(ware_0), 10.0);
        assert!(cargo.add(ware_0, 2.0).is_ok());
        assert!(cargo.add_all(&vec![WareAmount::new(ware_0, 2.0)]).is_ok());
        assert_eq!(cargo.get_amount(ware_0), 4.0);
        assert_eq!(cargo.add_to_max(ware_0, 20.0), 6.0);
    }

    #[test]
    fn test_cargo_whitelist_should_split_cargo_even() {
        let (ware_0, ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.set_whitelist(vec![ware_0, ware_1]);

        for ware_id in vec![ware_0, ware_1] {
            assert_eq!(cargo.free_volume(ware_id), 5.0);
            assert!(cargo.add(ware_id, 2.0).is_ok());
            assert!(cargo.add_all(&vec![WareAmount::new(ware_id, 2.0)]).is_ok());
            assert_eq!(cargo.get_amount(ware_id), 4.0);
            assert_eq!(cargo.free_volume(ware_id), 1.0);
            assert_eq!(cargo.add_to_max(ware_id, 20.0), 1.0);
            assert_eq!(cargo.get_amount(ware_id), 5.0);
            assert_eq!(cargo.free_volume(ware_id), 0.0);
            assert!(cargo.add(ware_id, 1.0).is_err());
        }
    }

    #[test]
    fn test_cargo_should_not_return_empty_lists() {
        let (ware_0, _ware_1, _ware_2) = create_wares();

        let mut cargo = Cargo::new(10.0);
        cargo.add(ware_0, 4.0).unwrap();
        cargo.remove(ware_0, 4.0).unwrap();

        assert!(cargo.get_wares_ids().next().is_none());
    }

    #[test]
    fn test_is_near_zero() {
        assert!(!is_near_zero(0.1));
        assert!(!is_near_zero(0.01));
        assert!(is_near_zero(0.005));
        assert!(is_near_zero(-0.005));
        assert!(!is_near_zero(-0.01));
    }
}
