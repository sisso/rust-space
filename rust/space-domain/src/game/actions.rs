///
/// Actions are setup by ActionRequest.
///
/// Systems:
/// - convert request into current actions
/// - execute actions
///
use specs::prelude::*;

use super::objects::ObjId;
use crate::game::actions::action_dock_system::DockSystem;
use crate::game::actions::action_jump_system::ActionJumpSystem;
use crate::game::actions::action_move_to_system::ActionMoveToSystem;
use crate::game::actions::action_request_handler_system::ActionRequestHandlerSystem;
use crate::game::actions::action_undock_system::UndockSystem;
use crate::game::sectors::JumpId;
use crate::utils::{DeltaTime, Position, Seconds, TotalTime};

mod action_dock_system;
mod action_jump_system;
mod action_move_to_system;
mod action_request_handler_system;
mod action_undock_system;
mod action_extract_system;

pub const ACTION_JUMP_TOTAL_TIME: DeltaTime = DeltaTime(2.0);

/// Not a component, but used to create requests
#[derive(Debug, Clone)]
pub enum Action {
    Undock,
    Jump { jump_id: JumpId },
    Dock { target_id: ObjId },
    MoveTo { pos: Position },
    Extract { target_id: ObjId },
}

impl Action {
    pub fn is_dock(&self) -> bool {
        match self {
            Action::Dock { .. } => true,
            _ => false,
        }
    }

    pub fn is_extract(&self) -> bool {
        match self {
            Action::Extract { .. } => true,
            _ => false,
        }
    }
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
pub struct ActionActive(pub Action);

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
pub struct ActionExtract;

#[derive(Debug, Clone, Component)]
pub struct ActionJump {
    complete_time: Option<TotalTime>,
}

impl ActionJump {
    pub fn new() -> Self {
        ActionJump {
            complete_time: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Actions {}

impl Actions {
    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        dispatcher.add(ActionRequestHandlerSystem, "action_request_handler", &[]);
        dispatcher.add(
            ActionMoveToSystem,
            "action_move_to",
            &["action_request_handler"],
        );
        dispatcher.add(DockSystem, "action_dock_to", &["action_request_handler"]);
        dispatcher.add(
            UndockSystem,
            "action_undock_to",
            &["action_request_handler"],
        );
        dispatcher.add(
            ActionJumpSystem,
            "action_jump_to",
            &["action_request_handler"],
        );
    }

    pub fn new() -> Self {
        Actions {}
    }
}
