use crate::game::extractables::Extractable;
use crate::game::locations::*;
use crate::game::objects::ObjId;
use crate::game::sectors::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct NewObj {
    pub speed: Option<Speed>,
    pub cargo_size: f32,
    pub extractable: Option<Extractable>,
    pub location: Option<Location>,
    pub can_dock: bool,
    pub has_dock: bool,
    pub ai: bool,
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
        self.location= Some(Location::Space { pos, sector_id });
        self
    }

    pub fn at_dock(mut self, docked_id: ObjId) -> Self {
        self.location= Some(Location::Dock { docked_id });
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
}
