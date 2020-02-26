use super::objects::ObjId;
use specs::prelude::*;
use std::collections::HashMap;

use crate::game::wares::WareId;
use crate::utils::*;

#[derive(Clone, Debug, Component)]
pub struct Extractable {
    pub ware_id: WareId,
}

pub struct Extractables;
