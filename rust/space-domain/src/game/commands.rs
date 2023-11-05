use std::collections::HashMap;

use specs::prelude::*;

use crate::game::locations::{EntityPerSectorIndex, Locations, INDEX_SECTOR_SYSTEM};
use crate::game::wares::{Cargos, WareId};
use crate::utils::*;

use super::actions::*;

use super::objects::*;
use super::sectors::*;

use crate::game::commands::command_trader_system::CommandTradeSystem;
use crate::game::order::TradeOrders;
use crate::game::{GameInitContext, RequireInitializer};
use command_mine_system::*;

pub mod command_mine_system;
pub mod command_trader_system;

#[derive(Debug, Clone)]
pub struct MineState {
    mine_target_id: Option<ObjId>,
    deliver_target_id: Option<ObjId>,
}

impl Default for MineState {
    fn default() -> Self {
        MineState {
            mine_target_id: None,
            deliver_target_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TradeState {
    Idle,
    PickUp {
        target_id: ObjId,
        wares: Vec<WareId>,
    },
    Deliver {
        target_id: ObjId,
        wares: Vec<WareId>,
    },
    Delay {
        deadline: TotalTime,
    },
}

impl Default for TradeState {
    fn default() -> Self {
        TradeState::Idle
    }
}

#[derive(Clone, Debug, Component)]
pub enum Command {
    Mine(MineState),
    Trade(TradeState),
}

impl Command {
    pub fn mine() -> Command {
        Command::Mine(Default::default())
    }

    pub fn as_mine(&self) -> Option<&MineState> {
        match self {
            Command::Mine(state) => Some(state),
            _ => None,
        }
    }

    pub fn trade() -> Command {
        Command::Trade(Default::default())
    }
}

pub struct Commands;

impl RequireInitializer for Commands {
    fn init(context: &mut GameInitContext) {
        context
            .dispatcher
            .add(CommandMineSystem, "command_mine", &[INDEX_SECTOR_SYSTEM]);

        context
            .dispatcher
            .add(CommandTradeSystem, "command_trade", &[INDEX_SECTOR_SYSTEM]);
    }
}

impl Commands {}

pub fn search_orders_target(
    sectors_index: &EntityPerSectorIndex,
    sector_id: SectorId,
    orders: &ReadStorage<TradeOrders>,
    wares_filter: Option<&Vec<WareId>>,
    already_targeting: Vec<ObjId>,
    to_pickup: bool,
) -> Option<(ObjId, Vec<WareId>)> {
    if to_pickup {
        assert!(
            wares_filter.is_none(),
            "pickup list of wares is not supported"
        );
    } else {
        let deliver_count = wares_filter.map(|v| v.len()).unwrap_or(0);
        assert!(
            deliver_count > 0,
            "deliver must define list of wares is not supported"
        );
    }

    let candidates = sectors_index.search_nearest_stations(sector_id).flat_map(
        |(_sector_id, distance, obj_id)| {
            let order = orders.get(obj_id).map(|orders| {
                if to_pickup {
                    orders.is_provide()
                } else {
                    orders.is_request_any(wares_filter.unwrap())
                }
            });

            match order {
                Some(true) => {
                    let active_traders =
                        already_targeting.iter().filter(|id| **id == obj_id).count() as u32;

                    let weight = distance + active_traders;
                    Some((weight, obj_id))
                }
                _ => None,
            }
        },
    );

    match crate::utils::next_lower(candidates) {
        Some(target_id) => {
            let wares = {
                let orders = orders.get(target_id).unwrap();
                if to_pickup {
                    orders.wares_provider()
                } else {
                    orders.wares_requests()
                }
            };

            let wares = wares
                .into_iter()
                .filter(|ware_id| wares_filter.is_none() || wares_filter.unwrap().contains(ware_id))
                .collect::<Vec<WareId>>();

            Some((target_id, wares))
        }
        _ => None,
    }
}
