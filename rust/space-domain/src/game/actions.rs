use specs::prelude::*;

use crate::utils::{Position, Seconds, DeltaTime};
use super::objects::ObjId;
use crate::game::sectors::JumpId;

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

}

#[derive(Debug, Clone, Component)]
pub struct ActionUndock;

#[derive(Debug, Clone, Component)]
pub struct ActionDock { target: ObjId }

#[derive(Debug, Clone, Component)]
pub struct ActionFly { to: Position }

#[derive(Debug, Clone, Component)]
pub struct ActionJump { jump_id: JumpId }

#[derive(Debug, Clone, Component)]
pub struct ActionMine { target: ObjId }

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
