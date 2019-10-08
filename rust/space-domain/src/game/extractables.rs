use std::collections::HashMap;

use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage};

use super::objects::{ObjId};
use crate::utils::*;

use crate::game::wares::WareId;
use crate::game::save::{Save, Load};
use crate::game::jsons::*;


#[derive(Clone,Debug)]
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

impl SpecComponent for Extractable {
    type Storage = HashMapStorage<Self>;
}

impl Extractables {
    pub fn new() -> Self {
        Extractables {
        }
    }

    pub fn init_world(world: &mut World) {
        world.register::<Extractable>();
    }
}
