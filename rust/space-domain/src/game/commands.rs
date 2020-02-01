use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use crate::game::extractables::Extractables;
use crate::game::locations::{ Locations, LocationSpace};
use crate::game::wares::Cargos;
use crate::utils::*;

use super::actions::*;
use super::objects::*;
use super::sectors::*;
use super::jsons;

mod command_mine_system;
use command_mine_system::*;
use std::borrow::BorrowMut;

#[derive(Debug, Clone, Component)]
pub struct CommandMine;

#[derive(Debug, Clone, Component)]
pub struct CommandMineTarget {
    target_obj_id: ObjId,
}

#[derive(Debug, Clone, Component)]
pub struct DeliverState {
    target_obj_id: ObjId,
}

pub struct Commands {
}

impl Commands {
    pub fn new() -> Self {
        Commands {
        }
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        dispatcher.add(SearchMineTargetsSystem, "command_mine_search_mine_targets", &["index_by_sector"]);
    }
}

pub fn set_command_mine(world: &mut World, entity: Entity) {
    let mut storage = world.write_storage::<CommandMine>();
    storage.borrow_mut().insert(entity, CommandMine).unwrap();

    info!("{:?} setting command to mine", entity);
}
