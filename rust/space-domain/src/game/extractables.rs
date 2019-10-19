use specs::prelude::*;
use std::collections::HashMap;
use super::objects::ObjId;

use crate::utils::*;
use crate::game::wares::WareId;

#[derive(Clone,Debug,Component)]
pub struct Extractable {
    pub ware_id: WareId,
    pub time: DeltaTime,
}

#[derive(Clone, Debug)]
struct State {
    extractable: Extractable
}

pub struct Extractables {
}

impl Extractables {
    pub fn new() -> Self {
        Extractables {
        }
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {

    }
}
