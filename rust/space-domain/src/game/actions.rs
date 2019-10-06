use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage, NullStorage};
use crate::utils::{V2, Position, Seconds, DeltaTime};
use super::objects::*;
use super::sectors::Sectors;
use super::Tick;
use std::collections::HashMap;
use crate::game::locations::Locations;
use crate::game::extractables::Extractables;
use crate::game::wares::Cargos;
use crate::game::jsons::JsonValueExtra;
use crate::game::events::Events;
use crate::game::sectors::JumpId;

pub const MIN_DISTANCE: f32 = 0.01;
pub const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

#[derive(Clone,Debug)]
pub struct ActionProgress {
    pub action_delay: DeltaTime,
}

impl SpecComponent for ActionProgress {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct HasAction;

impl SpecComponent for HasAction {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct ActionUndock;

impl SpecComponent for ActionUndock {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct ActionDock { target: ObjId }

impl SpecComponent for ActionDock {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct ActionFly { to: Position }

impl SpecComponent for ActionFly {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct ActionJump { jump_id: JumpId }

impl SpecComponent for ActionJump {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct ActionMine { target: ObjId }

impl SpecComponent for ActionMine {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone,Debug)]
pub struct Actions {

}

impl Actions {
    pub fn init_world(world: &mut World) {
        world.register::<HasAction>();
        world.register::<ActionUndock>();
        world.register::<ActionDock>();
        world.register::<ActionFly>();
        world.register::<ActionJump>();
        world.register::<ActionMine>();
        world.register::<ActionProgress>();
    }

    pub fn new() -> Self {
        Actions {
        }
    }
}
