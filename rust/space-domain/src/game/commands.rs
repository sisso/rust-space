use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use crate::game::extractables::Extractables;
use crate::game::locations::{ Locations, LocationSpace};
use crate::game::wares::Cargos;
use crate::utils::*;

use super::actions::*;
use super::objects::*;
use super::sectors::*;
use super::Tick;
use super::jsons;

//mod executor_command_idle;
//mod executor_command_mine;

mod command_mine_system;
use command_mine_system::*;

#[derive(Debug, Clone, Component)]
pub enum Command {
    Mine
}

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

struct CommandsMineSystems {
    search_targets_system: SearchMineTargetsSystem,
}

pub struct Commands {
    command_mine: CommandsMineSystems,
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            command_mine: CommandsMineSystems {
                search_targets_system: SearchMineTargetsSystem,
            }
        }
    }

    pub fn init_world(world: &mut World) {
        world.register::<Command>();
        world.register::<CommandMine>();
        world.register::<CommandMineTarget>();
        world.register::<DeliverState>();
    }

    pub fn execute(&mut self, world: &mut World) {
        self.command_mine.search_targets_system.run_now(world);
    }
}

