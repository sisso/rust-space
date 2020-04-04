use specs::prelude::*;
use crate::game::wares::WareId;
use crate::game::{RequireInitializer, GameInitContext};

#[derive(Debug, Clone)]
pub enum Order {
    WareProvide {
        wares_id: Vec<WareId>
    },

    WareRequest {
        wares_id: Vec<WareId>
    }
}

impl Order {
    pub fn add_wares_request(&self, buffer: &mut Vec<WareId>) {
        match self {
            Order::WareRequest { wares_id } =>
                buffer.extend(wares_id.iter()),
            _ => {},
        }
    }

    pub fn add_wares_provide(&self, mut buffer: Vec<WareId>) -> Vec<WareId> {
        match self {
            Order::WareProvide { wares_id } => buffer.extend(wares_id),
            _ => {},
        }
        buffer
    }

    pub fn is_provide(&self) -> bool {
        match self {
            Order::WareProvide { .. } => true,
            _ => false,
        }
    }

    pub fn request_any(&self, wares: &Vec<WareId>) -> Vec<WareId> {
        match self {
            Order::WareRequest { wares_id } =>
                wares_id.iter()
                    .filter(|ware_id| wares.contains(ware_id))
                    .copied()
                    .collect(),
            _ => Vec::new(),
        }
    }

    pub fn is_request_any(&self, wares: &Vec<WareId>) -> bool {
        match self {
            Order::WareRequest { wares_id } =>
                wares_id.iter()
                    .any(|ware_id| wares.contains(ware_id)),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Orders(pub Vec<Order>);

impl Orders {
    pub fn new(order: Order) -> Orders {
        Orders(vec![order])
    }

    pub fn wares_requests(&self) -> Vec<WareId> {
        let mut requests = vec![];
        for order in &self.0 {
            order.add_wares_request(&mut requests);
        }
        requests
    }

    pub fn wares_provider(&self) -> Vec<WareId> {
        self.0.iter().fold(vec![], |acc, i| {
            i.add_wares_provide(acc)
        })
    }

    pub fn is_provide(&self) -> bool {
        self.0.iter().any(|order| order.is_provide())
    }

    pub fn request_any(&self, wares: &Vec<WareId>) -> Vec<WareId> {
        self.wares_requests()
            .into_iter()
            .filter(|ware_id| wares.contains(ware_id))
            .collect()
    }

    pub fn is_request_any(&self, wares: &Vec<WareId>) -> bool {
        self.0.iter().any(|order| order.is_request_any(wares))
    }
}

impl RequireInitializer for Orders {
    fn init(context: &mut GameInitContext) {
        context.world.register::<Orders>();
    }
}