use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use crate::game::extractables::Extractables;
use crate::game::locations::{ Locations, LocationSpace};
use crate::game::save::{Load, Save, CanSave, CanLoad};
use crate::game::wares::Cargos;
use crate::utils::*;

use super::actions::*;
use super::objects::*;
use super::sectors::*;
use super::Tick;
use super::jsons;
use serde_json::Value;
use crate::game::jsons::JsonValueExtra;

//mod executor_command_idle;
//mod executor_command_mine;

mod command_mine_system;

#[derive(Debug, Clone, Component)]
pub struct HasCommand;

#[derive(Debug, Clone, Component)]
pub struct CommandMine;

#[derive(Debug, Clone, Component)]
pub struct MineState {
    // TODO: remove?
    mining: bool,
    target_obj_id: ObjId,
}

#[derive(Debug, Clone, Component)]
pub struct DeliverState {
    target_obj_id: ObjId,
}

#[derive(Debug, Clone)]
enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump { jump_id: JumpId },
    Dock { target: ObjId },
}

#[derive(Debug, Clone, Component)]
struct NavigationState {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: V2,
    path: VecDeque<NavigationStateStep>
}

impl NavigationState {
    fn is_complete(&self) -> bool {
        self.path.is_empty()
    }

//    fn navigation_next_action(&mut self) -> Action {
//        match self.path.pop_front() {
//            Some(NavigationStateStep::MoveTo { pos}) => {
//                Action::Fly { to: pos }
//            },
//            Some(NavigationStateStep::Jump { jump_id }) => {
//                Action::Jump { jump_id }
//            },
//            Some(NavigationStateStep::Dock { target }) => {
//                Action::Dock { target }
//            },
//            None => Action::Idle,
//        }
//    }

//    fn append_dock_at(&mut self, target: ObjId) {
//        self.path.push_back(NavigationStateStep::Dock {
//            target
//        })
//    }
}

pub struct Commands {
    command_mine_system: command_mine_system::CommandMineSystem
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            command_mine_system: command_mine_system::CommandMineSystem,
        }
    }

    pub fn init_world(world: &mut World) {
        world.register::<HasCommand>();
        world.register::<CommandMine>();
        world.register::<MineState>();
        world.register::<DeliverState>();
        world.register::<NavigationState>();
    }

    pub fn execute(&mut self, world: &mut World) {
        self.command_mine_system.run_now(world);
    }
}

