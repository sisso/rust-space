use specs::prelude::*;

use crate::utils::{Position, Seconds, DeltaTime};
use super::objects::ObjId;
use crate::game::sectors::JumpId;

mod action_request_handler_system;
mod action_undock_system;

#[derive(Debug, Clone, Component)]
pub enum ActionRequest {
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
pub struct Action {
    request: ActionRequest
}

#[derive(Debug, Clone, Component)]
pub struct ActionUndock;

#[derive(Debug, Clone, Component)]
pub struct ActionDock;

#[derive(Debug, Clone, Component)]
pub struct ActionMoveTo;

#[derive(Debug, Clone, Component)]
pub struct ActionJump;

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
