use crate::game::save::MapEntity;
///
/// Actions are setup by ActionRequest.
///
/// Systems:
/// - convert request into current actions
/// - execute actions
///
use bevy_ecs::prelude::*;
use commons::math::P2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::objects::ObjId;
use crate::game::sectors::JumpId;
use crate::game::utils::{DeltaTime, TotalTime};
use crate::game::wares::WareId;

pub mod action_dock_system;
pub mod action_extract_system;
pub mod action_jump_system;
pub mod action_move_to_system;
pub mod action_progress_system;
pub mod action_request_handler_system;
pub mod action_undock_system;
pub mod actions_system;

pub const ACTION_JUMP_TOTAL_TIME: DeltaTime = DeltaTime(2.0);

/// Not a component, but used to create requests and referenced by ActionActive component
/// to indicate what action is current active
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    Deorbit,
    Orbit {
        target_id: ObjId,
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

impl MapEntity for Action {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        match self {
            Action::Jump { jump_id } => {
                *jump_id = entity_map[jump_id];
            }
            Action::Dock { target_id } => {
                *target_id = entity_map[target_id];
            }
            Action::MoveToTargetPos { target_id, .. } => {
                *target_id = entity_map[target_id];
            }
            Action::Extract { target_id, ware_id } => {
                *target_id = entity_map[target_id];
                *ware_id = entity_map[ware_id];
            }
            Action::Orbit { target_id } => {
                *target_id = entity_map[target_id];
            }
            _ => {}
        }
    }
}

impl MapEntity for ActionRequest {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.0.map_entity(entity_map);
    }
}

impl MapEntity for ActionActive {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.0.map_entity(entity_map);
    }
}

/// Waiting time until ActiveAction can be completed
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ActionProgress {
    pub complete_time: TotalTime,
}

/// Request to change entity action
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ActionRequest(pub Action);

impl ActionRequest {
    pub fn get_action(&self) -> &Action {
        &self.0
    }
}

/// Current action that entity is doing, it is the source of truth. Others sidecart components can
/// to hold state or route into proper system
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Component, Serialize, Deserialize, Default)]
pub struct ActionUndock {}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Default)]
pub struct ActionDock {}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Default)]
pub struct ActionMoveTo {}

#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct ActionExtract {
    // accumulate the rest of extraction that is not enough to fill one volume unit between
    // runs, once get above 1, it should be deducted and added to cargo by the system
    pub rest_acc: f32,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ActionJump {
    complete_time: Option<TotalTime>,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ActionGeneric {}

impl ActionJump {
    pub fn new() -> Self {
        ActionJump {
            complete_time: None,
        }
    }
}

pub struct Actions;
