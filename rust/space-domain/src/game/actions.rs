use commons::math::P2;
///
/// Actions are setup by ActionRequest.
///
/// Systems:
/// - convert request into current actions
/// - execute actions
///
use specs::prelude::*;
use specs::Component;

use super::objects::ObjId;
use crate::game::actions::action_dock_system::DockSystem;
use crate::game::actions::action_extract_system::ActionExtractSystem;
use crate::game::actions::action_jump_system::ActionJumpSystem;
use crate::game::actions::action_move_to_system::ActionMoveToSystem;
use crate::game::actions::action_progress_system::ActionProgressSystem;
use crate::game::actions::action_request_handler_system::ActionRequestHandlerSystem;
use crate::game::actions::action_undock_system::UndockSystem;
use crate::game::sectors::JumpId;
use crate::game::wares::WareId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::{DeltaTime, TotalTime};

mod action_dock_system;
mod action_extract_system;
mod action_jump_system;
mod action_move_to_system;
mod action_progress_system;
mod action_request_handler_system;
mod action_undock_system;

pub const ACTION_JUMP_TOTAL_TIME: DeltaTime = DeltaTime(2.0);

/// Not a component, but used to create requests and referenced by ActionActive component
/// to indicate what action is current active
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Undock,
    Jump {
        jump_id: JumpId,
    },
    Dock {
        target_id: ObjId,
    },
    // move to a position in the same sector
    MoveTo {
        pos: P2,
    },
    // move to object in the same sector
    MoveToTargetPos {
        target_id: ObjId,
        last_position: Option<P2>,
    },
    Extract {
        target_id: ObjId,
        ware_id: WareId,
    },
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

/// Waiting time until ActiveAction can be completed
#[derive(Debug, Clone, Component)]
pub struct ActionProgress {
    pub complete_time: TotalTime,
}

/// Request to change entity action
#[derive(Debug, Clone, Component)]
pub struct ActionRequest(pub Action);

impl ActionRequest {
    pub fn get_action(&self) -> &Action {
        &self.0
    }
}

/// Current action that entity is doing
#[derive(Debug, Clone, Component)]
pub struct ActionActive(pub Action);

impl ActionActive {
    pub fn get_action(&self) -> &Action {
        &self.0
    }

    pub fn get_action_mut(&mut self) -> &mut Action {
        &mut self.0
    }
}

//
// actions markers
//
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

pub struct Actions;

const ACTION_PROGRESS_SYSTEM_NAME: &str = "action_progress_system";
const ACTION_REQUEST_SYSTEM_NAME: &str = "action_request_handler";

///
/// Flow:
/// - execute action progress
/// - execute request handler
/// - execute actions
impl RequireInitializer for Actions {
    fn init(context: &mut GameInitContext) {
        context
            .dispatcher
            .add(ActionProgressSystem, ACTION_PROGRESS_SYSTEM_NAME, &[]);
        context.dispatcher.add(
            ActionRequestHandlerSystem,
            ACTION_REQUEST_SYSTEM_NAME,
            &[ACTION_PROGRESS_SYSTEM_NAME],
        );

        let default_dependencies = [ACTION_PROGRESS_SYSTEM_NAME, ACTION_REQUEST_SYSTEM_NAME];

        context
            .dispatcher
            .add(ActionMoveToSystem, "action_move_to", &default_dependencies);
        context
            .dispatcher
            .add(DockSystem, "action_dock_to", &["action_request_handler"]);
        context
            .dispatcher
            .add(UndockSystem, "action_undock_to", &default_dependencies);
        context
            .dispatcher
            .add(ActionJumpSystem, "action_jump_to", &default_dependencies);
        context
            .dispatcher
            .add(ActionExtractSystem, "action_extract", &default_dependencies);
    }
}
