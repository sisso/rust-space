use specs::prelude::*;

use crate::game::extractables::Extractable;
use crate::game::locations::*;
use crate::game::objects::ObjId;
use crate::game::sectors::*;
use crate::utils::*;

#[derive(Debug, Clone, Component)]
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
    pub command_mine: bool,
    pub shipyard: bool,
}

impl NewObj {
    pub fn new() -> NewObj {
        NewObj {
            speed: None,
            cargo_size: 0.0,
            extractable: None,
            location: None,
            can_dock: false,
            has_dock: false,
            ai: false,
            station: false,
            sector: false,
            jump_to: None,
            command_mine: false,
            shipyard: false,
        }
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
        self.sector= true;
        self
    }

    pub fn with_jump(mut self, jump_to: Entity) -> Self {
        self.jump_to = Some(jump_to);
        self
    }

    pub fn with_command_mine(mut self) -> Self {
        self.command_mine = true;
        self
    }

    pub fn with_shipyard(mut self) -> Self {
        self.shipyard = true;
        self
    }
}
