use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

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
    orders_by_id: Vec<(TradeOrderId, TradeOrdersEntry)>,
}

// impl std::fmt::Debug for TradeOrders {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut l = f.debug_list();
//         for (id, orders) in self.orders_by_id {
//             l.entry((id, orders));
//         }
//         l.finish()
//     }
// }
//

#[derive(Debug, Clone, PartialEq)]
struct TradeOrdersEntry {
    provided: HashSet<WareId>,
    requested: HashSet<WareId>,
}

impl TradeOrders {
    pub fn from_provided(id: TradeOrderId, provided: &[WareId]) -> Self {
        let entry = TradeOrdersEntry {
            provided: provided.iter().copied().collect(),
            requested: Default::default(),
        };

        TradeOrders {
            orders_by_id: vec![(id, entry)],
        }
    }

    pub fn from_requested(id: TradeOrderId, requested: &[WareId]) -> Self {
        let entry = TradeOrdersEntry {
            provided: Default::default(),
            requested: requested.iter().copied().collect(),
        };

        TradeOrders {
            orders_by_id: vec![(id, entry)],
        }
    }

    pub fn is_empty(&self) -> bool {
        for (_, v) in self.orders_by_id {
            if !v.provided.is_empty() || !v.requested.is_empty() {
                return false;
            }
        }
        true
    }

    pub fn wares_requests(&self) -> Vec<WareId> {
        self.orders_by_id
            .iter()
            .flat_map(|(_, entry)| entry.requested.iter())
            .cloned()
            .collect()
    }

    pub fn wares_provider(&self) -> Vec<WareId> {
        self.orders_by_id
            .iter()
            .flat_map(|(_, entry)| entry.provided.iter())
            .cloned()
            .collect()
    }

    pub fn is_provide(&self) -> bool {
        for (_, v) in self.orders_by_id {
            if !v.provided.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn is_requesting(&self) -> bool {
        for (_, v) in self.orders_by_id {
            if !v.requested.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn is_requesting_ware(&self, wares: WareId) -> bool {
        for (_, e) in &self.orders_by_id {}
    }

    pub fn request_any(&self, wares: &[WareId]) -> Vec<WareId> {
        wares
            .iter()
            .copied()
            .filter(|ware_id| self.requested.contains(ware_id))
            .collect()
    }

    pub fn is_request_exactly(&self, wares: &[WareId]) -> bool {
        self.request_any(wares).len() == wares.len()
    }

    pub fn is_request_any(&self, wares: &[WareId]) -> bool {
        !self.request_any(wares).is_empty()
    }

    pub fn add_request(&mut self, id: TradeOrderId, ware_id: WareId) {
        self.requested.insert(ware_id);
    }

    pub fn add_provider(&mut self, id: TradeOrderId, ware_id: WareId) {
        self.provided.insert(ware_id);
    }

    pub fn remove_request(&mut self, id: TradeOrderId, ware_id: WareId) {
        todo!()
    }

    pub fn remove_provider(&mut self, id: TradeOrderId, ware_id: WareId) {
        todo!()
    }
}

impl RequireInitializer for TradeOrders {
    fn init(context: &mut GameInitContext) {
        context.world.register::<TradeOrders>();
    }
}
