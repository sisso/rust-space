use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;

use specs::prelude::*;
use specs::Entity;

use commons::math::P2;
use space_domain::game::actions::Action;
use space_domain::game::factory::Factory;
use space_domain::game::locations::{Location, LocationSpace, Locations};
use space_domain::game::navigations::NavigationMoveTo;
use space_domain::game::order::TradeOrders;
use space_domain::game::sectors::Jump;
use space_domain::game::shipyard::{ProductionOrder, Shipyard};
use space_domain::game::wares::Cargo;
use space_domain::game::{events, Game};

use super::{decode_entity, encode_entity};

pub type Id = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
}

#[derive(Clone, Debug)]
pub struct ObjKind {
    pub(crate) fleet: bool,
    pub(crate) jump: bool,
    pub(crate) station: bool,
    pub(crate) asteroid: bool,
    pub(crate) astro: bool,
    pub(crate) astro_star: bool,
    pub(crate) factory: bool,
    pub(crate) shipyard: bool,
}

#[derive(Clone, Debug)]
pub struct ObjOrbitData {
    pub(crate) radius: f32,
    pub(crate) parent_pos: P2,
}

impl ObjOrbitData {
    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_parent_pos(&self) -> (f32, f32) {
        (self.parent_pos.x, self.parent_pos.y)
    }
}

#[derive(Clone, Debug)]
pub struct ObjCoords {
    pub(crate) location: LocationSpace,
    pub(crate) is_docked: bool,
}

impl ObjCoords {
    pub fn get_sector_id(&self) -> Id {
        encode_entity(self.location.sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        (self.location.pos.x, self.location.pos.y)
    }

    pub fn is_docked(&self) -> bool {
        self.is_docked
    }
}

#[derive(Clone, Debug)]
pub struct ObjData {
    pub(crate) id: Entity,
    pub(crate) coords: P2,
    pub(crate) sector_id: Entity,
    pub(crate) docked: Option<Entity>,
    pub(crate) kind: ObjKind,
    pub(crate) orbit: Option<ObjOrbitData>,
    pub(crate) trade_orders: Vec<ObjTradeOrder>,
}

impl ObjData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.id)
    }

    pub fn is_docked(&self) -> bool {
        self.docked.is_some()
    }

    pub fn get_docked_id(&self) -> Option<Id> {
        self.docked.map(|e| encode_entity(e))
    }

    pub fn get_sector_id(&self) -> Id {
        encode_entity(self.sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        (self.coords.x, self.coords.y)
    }

    pub fn get_orbit(&self) -> Option<ObjOrbitData> {
        self.orbit.clone()
    }

    pub fn is_fleet(&self) -> bool {
        self.kind.fleet
    }

    pub fn is_station(&self) -> bool {
        self.kind.station
    }

    pub fn is_asteroid(&self) -> bool {
        self.kind.asteroid
    }

    pub fn is_jump(&self) -> bool {
        self.kind.jump
    }

    pub fn is_astro(&self) -> bool {
        self.kind.astro
    }

    pub fn is_astro_star(&self) -> bool {
        self.kind.astro_star
    }

    pub fn is_factory(&self) -> bool {
        self.kind.factory
    }

    pub fn is_shipyard(&self) -> bool {
        self.kind.shipyard
    }

    pub fn get_trade_orders(&self) -> Vec<ObjTradeOrder> {
        self.trade_orders.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ObjActionKind {
    Undock,
    Jump,
    Dock,
    MoveTo,
    MoveToTargetPos,
    Extract,
}

#[derive(Clone, Debug)]
pub struct ObjAction {
    action: Action,
}

impl ObjAction {
    pub fn get_kind(&self) -> ObjActionKind {
        match &self.action {
            Action::Undock => ObjActionKind::Undock,
            Action::Jump { .. } => ObjActionKind::Jump,
            Action::Dock { .. } => ObjActionKind::Dock,
            Action::MoveTo { .. } => ObjActionKind::MoveTo,
            Action::MoveToTargetPos { .. } => ObjActionKind::MoveToTargetPos,
            Action::Extract { .. } => ObjActionKind::Extract,
        }
    }

    pub fn get_target(&self) -> Option<Id> {
        match &self.action {
            Action::Jump { jump_id } => Some(encode_entity(*jump_id)),
            Action::Dock { target_id } => Some(encode_entity(*target_id)),
            Action::MoveToTargetPos { target_id, .. } => Some(encode_entity(*target_id)),
            Action::Extract { target_id, .. } => Some(encode_entity(*target_id)),
            _ => None,
        }
    }

    pub fn get_pos(&self) -> Option<(f32, f32)> {
        match &self.action {
            _ => None,
            Action::MoveTo { pos } => Some((pos.x, pos.y)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjDesc {
    pub(crate) id: Id,
    pub(crate) label: String,
    pub(crate) extractable: Option<Entity>,
    pub(crate) action: Option<Action>,
    pub(crate) nav_move_to: Option<NavigationMoveTo>,
    pub(crate) cargo: Option<Cargo>,
    pub(crate) factory: Option<Factory>,
    pub(crate) shipyard: Option<Shipyard>,
    pub(crate) docked_fleets: Vec<Id>,
}

impl ObjDesc {
    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_label(&self) -> &str {
        self.label.as_str()
    }

    pub fn get_action(&self) -> Option<ObjAction> {
        self.action.as_ref().map(|action| ObjAction {
            action: action.clone(),
        })
    }

    pub fn get_nav_move_to_target(&self) -> Option<Id> {
        self.nav_move_to
            .as_ref()
            .map(|i| encode_entity(i.target_id))
    }

    pub fn get_cargo(&self) -> Option<ObjCargo> {
        self.cargo.as_ref().map(|cargo| ObjCargo {
            cargo: cargo.clone(),
        })
    }

    pub fn get_factory(&self) -> Option<ObjFactory> {
        self.factory.as_ref().map(|factory| ObjFactory {
            factory: factory.clone(),
        })
    }

    pub fn get_shipyard(&self) -> Option<ObjShipyard> {
        self.shipyard.as_ref().map(|shipyard| ObjShipyard {
            shipyard: shipyard.clone(),
        })
    }

    pub fn get_docked_fleets(&self) -> Vec<u64> {
        self.docked_fleets.clone()
    }
}

#[derive(Clone, Debug)]
pub struct ObjCargo {
    cargo: Cargo,
}

impl ObjCargo {
    pub fn volume_total(&self) -> u32 {
        self.cargo.get_current_volume()
    }
    pub fn volume_max(&self) -> u32 {
        self.cargo.get_max()
    }
    pub fn get_wares(&self) -> Vec<(Id, u32)> {
        self.cargo
            .get_wares()
            .iter()
            .map(|i| (encode_entity(i.ware_id), i.amount))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct SectorData {
    pub(crate) id: Id,
    pub(crate) coords: (i32, i32),
    pub(crate) label: String,
}

impl SectorData {
    pub fn get_id(&self) -> Id {
        self.id
    }
    pub fn get_coords(&self) -> (i32, i32) {
        self.coords.clone()
    }

    pub fn get_label(&self) -> &str {
        self.label.as_str()
    }
}

#[derive(Clone)]
pub struct JumpData {
    pub(crate) entity: Entity,
    pub(crate) game: Rc<RefCell<Game>>,
}

impl JumpData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.entity)
    }

    pub fn get_sector_id(&self) -> Id {
        let g = self.game.borrow();
        let locations = g.world.read_storage::<Location>();
        let loc = Locations::resolve_space_position(&locations, self.entity);
        encode_entity(loc.unwrap().sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        let g = self.game.borrow();
        let locations = g.world.read_storage::<Location>();
        let loc = Locations::resolve_space_position(&locations, self.entity);
        let pos = loc.unwrap().pos;
        (pos.x, pos.y)
    }

    pub fn get_to_sector_id(&self) -> Id {
        let g = self.game.borrow();
        let jumps = g.world.read_storage::<Jump>();
        encode_entity((&jumps).get(self.entity).unwrap().target_sector_id)
    }

    pub fn get_to_coords(&self) -> (f32, f32) {
        let g = self.game.borrow();
        let jumps = g.world.read_storage::<Jump>();
        let pos = (&jumps).get(self.entity).unwrap().target_pos;
        (pos.x, pos.y)
    }
}

pub struct ObjFactory {
    pub(crate) factory: Factory,
}

impl ObjFactory {
    pub fn is_producing(&self) -> bool {
        self.factory.production_time.is_some()
    }

    pub fn get_receipt_label(&self) -> &str {
        self.factory.production.label.as_str()
    }
}

pub struct ObjShipyard {
    pub(crate) shipyard: Shipyard,
}

impl ObjShipyard {
    pub fn is_producing(&self) -> bool {
        self.shipyard.is_producing()
    }
    pub fn get_producing_prefab_id(&self) -> Option<u64> {
        self.shipyard.get_producing().map(|id| encode_entity(id))
    }

    pub fn get_order(&self) -> Option<Id> {
        match self.shipyard.order {
            ProductionOrder::None => None,
            ProductionOrder::Next(id) => Some(encode_entity(id)),
            ProductionOrder::Random => None,
            ProductionOrder::RandomSelected(id) => Some(encode_entity(id)),
        }
    }
}

#[derive(Debug)]
pub struct WareData {
    pub(crate) id: Id,
    pub(crate) label: String,
}

impl WareData {
    pub fn get_id(&self) -> Id {
        self.id
    }
    pub fn get_label(&self) -> &str {
        self.label.as_str()
    }
}

#[derive(Debug)]
pub struct EventData {
    pub(crate) event: events::Event,
}

impl EventData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.event.id)
    }

    pub fn get_kind(&self) -> EventKind {
        match self.event.kind {
            events::EventKind::Add => EventKind::Add,
            events::EventKind::Move => EventKind::Move,
            events::EventKind::Jump => EventKind::Jump,
            events::EventKind::Dock => EventKind::Dock,
            events::EventKind::Undock => EventKind::Undock,
        }
    }
}

#[derive(Debug)]
pub struct PrefabData {
    pub(crate) id: Id,
    pub(crate) label: String,
}

impl PrefabData {
    pub fn get_id(&self) -> Id {
        self.id
    }
    pub fn get_label(&self) -> &str {
        self.label.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct ObjTradeOrder {
    pub(crate) request: bool,
    pub(crate) provide: bool,
    pub(crate) ware_id: Id,
}

impl ObjTradeOrder {
    pub fn is_request(&self) -> bool {
        self.request
    }
    pub fn is_provide(&self) -> bool {
        self.provide
    }
    pub fn get_ware(&self) -> Id {
        self.ware_id
    }
}
