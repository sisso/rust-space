use itertools::Itertools;
use specs::prelude::*;

use crate::game::wares::WareId;
use crate::game::{GameInitContext, RequireInitializer};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TradeOrderId(u16);

pub const TRADE_ORDER_ID_SHIPYARD: TradeOrderId = TradeOrderId(0);
pub const TRADE_ORDER_ID_FACTORY: TradeOrderId = TradeOrderId(1);
pub const TRADE_ORDER_ID_EXTRACTABLE: TradeOrderId = TradeOrderId(2);
pub const TRADE_ORDER_ID_BUILDING_SITE: TradeOrderId = TradeOrderId(3);

#[derive(Clone, Debug, Component, Default, PartialEq)]
pub struct TradeOrders {
    provided: Vec<(TradeOrderId, WareId)>,
    requested: Vec<(TradeOrderId, WareId)>,
}

impl TradeOrders {
    pub fn from_provided(order_id: TradeOrderId, provided: &[WareId]) -> Self {
        TradeOrders {
            provided: provided
                .iter()
                .copied()
                .map(|ware_id| (order_id, ware_id))
                .collect(),
            requested: vec![],
        }
    }

    pub fn from_requested(order_id: TradeOrderId, requested: &[WareId]) -> Self {
        TradeOrders {
            requested: requested
                .iter()
                .copied()
                .map(|ware_id| (order_id, ware_id))
                .collect(),
            provided: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.provided.is_empty() && self.requested.is_empty()
    }

    pub fn wares_requests(&self) -> Vec<WareId> {
        let wares: Vec<WareId> = self.requested.iter().map(|(_, ware_id)| *ware_id).collect();
        wares.into_iter().unique().collect()
    }

    pub fn wares_provider(&self) -> Vec<WareId> {
        let wares: Vec<WareId> = self.provided.iter().map(|(_, ware_id)| *ware_id).collect();
        wares.into_iter().unique().collect()
    }

    pub fn is_provide(&self) -> bool {
        !self.provided.is_empty()
    }

    pub fn is_requesting(&self) -> bool {
        !self.requested.is_empty()
    }

    pub fn is_requesting_ware(&self, ware_id: WareId) -> bool {
        self.requested.iter().find(|i| i.1 == ware_id).is_some()
    }

    pub fn is_providing_ware(&self, ware_id: WareId) -> bool {
        self.provided.iter().find(|i| i.1 == ware_id).is_some()
    }

    pub fn request_any(&self, wares: &[WareId]) -> Vec<WareId> {
        wares
            .iter()
            .copied()
            .filter(|ware_id| self.is_requesting_ware(*ware_id))
            .collect()
    }

    pub fn is_request_exactly(&self, wares: &[WareId]) -> bool {
        self.request_any(wares).len() == wares.len()
    }

    pub fn is_request_any(&self, wares: &[WareId]) -> bool {
        !self.request_any(wares).is_empty()
    }

    pub fn add_request(&mut self, order_id: TradeOrderId, ware_id: WareId) {
        if self
            .requested
            .iter()
            .find(|(i_order_id, i_ware_id)| order_id == *i_order_id && *i_ware_id == ware_id)
            .is_some()
        {
            return;
        }

        self.requested.push((order_id, ware_id));

        log::debug!("trade order updated by add_request {:?}", self);
    }

    pub fn add_provider(&mut self, order_id: TradeOrderId, ware_id: WareId) {
        if self
            .provided
            .iter()
            .find(|(i_order_id, i_ware_id)| order_id == *i_order_id && *i_ware_id == ware_id)
            .is_some()
        {
            return;
        }

        self.provided.push((order_id, ware_id));
        log::debug!("trade order updated by add_provide {:?}", self);
    }

    pub fn remove_request(&mut self, order_id: TradeOrderId, ware_id: WareId) {
        self.provided
            .retain(|(i_order_id, i_ware_id)| order_id != *i_order_id || *i_ware_id == ware_id);
        log::debug!("trade order updated by remove_request {:?}", self);
    }

    pub fn remove_provider(&mut self, order_id: TradeOrderId, ware_id: WareId) {
        self.provided
            .retain(|(i_order_id, i_ware_id)| order_id != *i_order_id || *i_ware_id == ware_id);
        log::debug!("trade order updated by remove_provide{:?}", self);
    }

    pub fn remove_by_id(&mut self, order_id: TradeOrderId) {
        self.provided
            .retain(|(i_order_id, _)| *i_order_id == order_id);
        self.provided
            .retain(|(i_order_id, _)| *i_order_id == order_id);
        log::debug!("trade order updated by remove_by_id {:?}", self);
    }
}

impl RequireInitializer for TradeOrders {
    fn init(context: &mut GameInitContext) {
        context.world.register::<TradeOrders>();
    }
}
