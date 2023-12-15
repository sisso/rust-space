use bevy_ecs::prelude::*;

use crate::game::wares::{ResourceAccessibility, WareId};

#[derive(Clone, Debug, Component)]
pub struct Extractable {
    pub ware_id: WareId,
    pub accessibility: ResourceAccessibility,
}
