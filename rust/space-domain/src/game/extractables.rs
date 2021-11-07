
use specs::prelude::*;


use crate::game::wares::WareId;


#[derive(Clone, Debug, Component)]
pub struct Extractable {
    pub ware_id: WareId,
}

pub struct Extractables;
