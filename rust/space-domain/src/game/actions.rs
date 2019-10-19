///
/// Actions are setup by ActionRequest.
///
/// Systems:
/// - convert request into current actions
/// - execute actions
///

use specs::prelude::*;

use crate::utils::{Position, Seconds, DeltaTime, TotalTime};
use super::objects::ObjId;
use crate::game::sectors::JumpId;

mod action_request_handler_system;
mod action_undock_system;
mod action_move_to_system;
mod action_jump_system;
mod action_dock_system;

pub const ACTION_JUMP_TOTAL_TIME: DeltaTime = DeltaTime(2.0);

#[derive(Debug, Clone)]
pub enum Action {
    Undock,
    Jump { jump_id: JumpId },
    Dock { target_id: ObjId },
    MoveTo { pos: Position }
}

#[derive(Debug, Clone, Component)]
pub struct ActionProgress {
    pub action_delay: DeltaTime,
}

#[derive(Debug, Clone, Component)]
pub struct ActionRequest(pub Action);

impl ActionRequest {
    pub fn get_action(&self) -> &Action {
        &self.0
    }
}

#[derive(Debug, Clone, Component)]
pub struct ActionActive(pub  Action);

impl ActionActive {
    pub fn get_action(&self) -> &Action {
        &self.0
    }
}
#[derive(Debug, Clone, Component)]
pub struct ActionUndock;

#[derive(Debug, Clone, Component)]
pub struct ActionDock;

#[derive(Debug, Clone, Component)]
pub struct ActionMoveTo;

#[derive(Debug, Clone, Component)]
pub struct ActionJump {
    complete_time: Option<TotalTime>,
}

impl ActionJump {
    pub fn new() -> Self {
        ActionJump { complete_time: None}
    }
}

#[derive(Debug, Clone, Component)]
pub struct ActionMine;

#[derive(Clone,Debug)]
pub struct Actions {

}

impl Actions {
    pub fn init_world(world: &mut World) {
    }

    pub fn new() -> Self {
        Actions {
        }
    }
}
