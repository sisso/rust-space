use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use crate::game::extractables::Extractables;
use crate::game::locations::{Location, Locations};
use crate::game::wares::Cargos;
use crate::utils::*;

use super::actions::*;
use super::jsons;
use super::objects::*;
use super::sectors::*;

mod command_mine_system;
use command_mine_system::*;
use std::borrow::BorrowMut;

#[derive(Debug, Clone, Component)]
pub struct CommandMine {
    mine_target_id: Option<ObjId>,
    deliver_target_id: Option<ObjId>,
}

impl CommandMine {
    pub fn new() -> Self {
        CommandMine {
            mine_target_id: None,
            deliver_target_id: None,
        }
    }
}

pub struct Commands {}

impl Commands {
    pub fn new() -> Self {
        Commands {}
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        dispatcher.add(
            CommandMineSystem,
            "command_mine_search_mine_targets",
            &["index_by_sector"],
        );
    }

    pub fn set_command_mine(world: &mut World, entity: Entity) {
        let mut storage = world.write_storage::<CommandMine>();
        storage
            .borrow_mut()
            .insert(entity, CommandMine::new())
            .unwrap();

        info!("{:?} setting command to mine", entity);
    }
}
