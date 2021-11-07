use specs::prelude::*;

use crate::game::commands::Command;
use crate::game::extractables::Extractable;
use crate::game::factory::Factory;
use crate::game::locations::*;
use crate::game::objects::ObjId;
use crate::game::sectors::*;
use crate::game::shipyard::Shipyard;
use crate::utils::*;

#[derive(Debug, Clone, Component, Default)]
pub struct NewObj {
    pub speed: Option<Speed>,
    pub cargo_size: f32,
    pub extractable: Option<Extractable>,
    pub location: Option<Location>,
    pub can_dock: bool,
    pub has_dock: bool,
    pub ai: bool,
    pub station: bool,
    pub sector: bool,
    pub jump_to: Option<Entity>,
    pub command: Option<Command>,
    pub shipyard: Option<Shipyard>,
    pub ware: bool,
    pub factory: Option<Factory>,
    pub label: Option<String>,
}

impl NewObj {
    pub fn new() -> NewObj {
        Default::default()
    }

    pub fn with_cargo(mut self, cargo: f32) -> Self {
        self.cargo_size = cargo;
        self
    }

    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn at_position(mut self, sector_id: SectorId, pos: Position) -> Self {
        self.location = Some(Location::Space { pos, sector_id });
        self
    }

    pub fn at_dock(mut self, docked_id: ObjId) -> Self {
        self.location = Some(Location::Dock { docked_id });
        self
    }

    pub fn extractable(mut self, extractable: Extractable) -> Self {
        self.extractable = Some(extractable);
        self
    }

    pub fn has_dock(mut self) -> Self {
        self.has_dock = true;
        self
    }

    pub fn can_dock(mut self) -> Self {
        self.can_dock = true;
        self
    }

    pub fn with_ai(mut self) -> Self {
        self.ai = true;
        self
    }

    pub fn as_station(mut self) -> Self {
        self.station = true;
        self
    }

    pub fn with_sector(mut self) -> Self {
        self.sector = true;
        self
    }

    pub fn with_jump(mut self, jump_to: Entity) -> Self {
        self.jump_to = Some(jump_to);
        self
    }

    pub fn with_command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    pub fn with_shipyard(mut self, shipyard: Shipyard) -> Self {
        self.shipyard = Some(shipyard);
        self
    }

    pub fn with_ware(mut self) -> Self {
        self.ware = true;
        self
    }

    pub fn with_factory(mut self, factory: Factory) -> Self {
        self.factory = Some(factory);
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}
