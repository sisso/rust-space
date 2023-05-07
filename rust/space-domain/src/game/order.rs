use std::collections::HashSet;

use specs::prelude::*;

use crate::game::wares::WareId;
use crate::game::{GameInitContext, RequireInitializer};

#[derive(Debug, Clone, Component, Default)]
pub struct Orders {
    provided: HashSet<WareId>,
    requested: HashSet<WareId>,
}

impl Orders {
    pub fn from_provided(provided: &[WareId]) -> Self {
        Orders {
            provided: provided.iter().copied().collect(),
            requested: Default::default(),
        }
    }

    pub fn from_requested(requested: &[WareId]) -> Self {
        Orders {
            provided: Default::default(),
            requested: requested.iter().copied().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.requested.is_empty() && self.provided.is_empty()
    }

    pub fn wares_requests(&self) -> Vec<WareId> {
        self.requested.iter().cloned().collect()
    }

    pub fn wares_provider(&self) -> Vec<WareId> {
        self.provided.iter().cloned().collect()
    }

    pub fn is_provide(&self) -> bool {
        !self.provided.is_empty()
    }

    pub fn request_any(&self, wares: &Vec<WareId>) -> Vec<WareId> {
        wares
            .iter()
            .copied()
            .filter(|ware_id| self.requested.contains(ware_id))
            .collect()
    }

    pub fn is_request_any(&self, wares: &Vec<WareId>) -> bool {
        !self.request_any(wares).is_empty()
    }

    pub fn add_request(&mut self, ware_id: WareId) {
        self.requested.insert(ware_id);
    }

    pub fn add_provider(&mut self, ware_id: WareId) {
        self.provided.insert(ware_id);
    }
}

impl RequireInitializer for Orders {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Orders>();
    }
}
