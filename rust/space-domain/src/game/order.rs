use specs::prelude::*;
use crate::game::wares::WareId;
use crate::game::{RequireInitializer, GameInitContext};

#[derive(Debug, Clone, Component)]
pub enum Order {
    WareRequest {
        wares_id: Vec<WareId>
    }
}

pub struct Orders;

impl RequireInitializer for Orders {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Order>();
    }
}