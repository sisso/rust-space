use std::collections::{HashMap, VecDeque};

use specs::prelude::*;

use crate::game::extractables::Extractables;
use crate::game::locations::{Location, Locations, INDEX_SECTOR_SYSTEM, EntityPerSectorIndex};
use crate::game::wares::{Cargos, WareId};
use crate::utils::*;

use super::actions::*;
use super::jsons;
use super::objects::*;
use super::sectors::*;

use command_mine_system::*;
use std::borrow::BorrowMut;
use crate::game::{RequireInitializer, GameInitContext};
use crate::game::commands::command_trader_system::CommandTradeSystem;
use crate::game::order::Orders;

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
pub struct TradeState {
    pub pickup_target_id: Option<ObjId>,
    pub deliver_target_id: Option<ObjId>,
    pub wares: Vec<WareId>,
}

impl Default for TradeState {
    fn default() -> Self {
        TradeState {
            pickup_target_id: None,
            deliver_target_id: None,
            wares: vec![],
        }
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
            _ =>  None
        }
    }

   pub fn trade() -> Command {
       Command::Trade(Default::default())
   }
}

pub struct Commands;

impl RequireInitializer for Commands {
    fn init(context: &mut GameInitContext) {
        context.dispatcher.add(
            CommandMineSystem,
            "command_mine",
            &[INDEX_SECTOR_SYSTEM],
        );

        context.dispatcher.add(
            CommandTradeSystem,
            "command_trade",
            &[INDEX_SECTOR_SYSTEM],
        );
    }
}

impl Commands {
    // pub fn set_command_mine(world: &mut World, entity: Entity) {
    //     world.write_storage::<Command>()
    //         .borrow_mut()
    //         .insert(entity, Command::Mine(CommandMine::new()))
    //         .unwrap();
    //
    //     info!("{:?} setting command to mine", entity);
    // }
}

pub fn search_orders_target(
    sectors_index: &EntityPerSectorIndex,
    sector_id: SectorId,
    orders: &ReadStorage<Orders>,
    wares_whitelist: Option<&Vec<WareId>>,
    requests: bool,
) -> Option<(ObjId, Vec<WareId>)> {
    let candidates = sectors_index.search_nearest_stations(sector_id);

    candidates.iter()
        .flat_map(|(sector_id, candidate_id)| {
            match orders.get(*candidate_id).map(|orders| {
                if requests {
                    orders.ware_requests()
                } else {
                    orders.wares_provider()
                }
            }) {
                Some(wares) if !wares.is_empty() => {
                    let is_valid = match wares_whitelist {
                        Some(whitelist) =>
                            whitelist.iter().any(|ware_id| wares.contains(ware_id)),
                        None => true,
                    };

                    if is_valid  {
                        Some((*candidate_id, wares))
                    } else {
                        None
                    }
                },
                _ => None,
            }
        })
        .next()    // find nearest deliver
}

