use commons::math::{Rad, P2};
use specs::prelude::*;

use crate::game::commands::Command;
use crate::game::extractables::Extractable;
use crate::game::factory::Factory;
use crate::game::locations::*;
use crate::game::objects::ObjId;
use crate::game::sectors::*;
use crate::game::shipyard::Shipyard;
use crate::game::wares::Volume;
use crate::utils::*;

#[derive(Debug, Clone, Component)]
pub struct NewObjOrbit {
    pub parent: ObjId,
    pub distance: f32,
    pub angle: Rad,
}

#[derive(Debug, Clone, Component, Default)]
pub struct NewObj {
    pub speed: Option<Speed>,
    pub cargo_size: Volume,
    pub extractable: Option<Extractable>,
    pub location: Option<Location>,
    pub can_dock: bool,
    pub fleet: bool,
    pub has_dock: bool,
    pub ai: bool,
    pub station: bool,
    pub sector: Option<P2>,
    pub jump_to: Option<(SectorId, P2)>,
    pub command: Option<Command>,
    pub shipyard: Option<Shipyard>,
    pub ware: bool,
    pub factory: Option<Factory>,
    pub label: Option<String>,
    pub code: Option<String>,
    pub pos: Option<V2>,
    pub star: Option<()>,
    pub planet: Option<()>,
    pub asteroid: Option<()>,
    pub orbit: Option<NewObjOrbit>,
}

impl NewObj {
    pub fn new() -> NewObj {
        Default::default()
    }

    pub fn with_cargo(mut self, cargo: Volume) -> Self {
        self.cargo_size = cargo;
        self
    }

    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn at_position(mut self, sector_id: SectorId, pos: P2) -> Self {
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

    pub fn with_station(mut self) -> Self {
        self.station = true;
        self
    }

    pub fn with_fleet(mut self) -> Self {
        self.fleet = true;
        self
    }

    pub fn with_star(mut self) -> Self {
        self.star = Some(());
        self
    }

    pub fn with_planet(mut self) -> Self {
        self.planet = Some(());
        self
    }

    pub fn with_asteroid(mut self) -> Self {
        self.asteroid = Some(());
        self
    }

    pub fn with_sector(mut self, pos: P2) -> Self {
        self.sector = Some(pos);
        self
    }

    pub fn with_jump(mut self, target_sector_id: SectorId, target_pos: P2) -> Self {
        self.jump_to = Some((target_sector_id, target_pos));
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

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_pos(mut self, pos: V2) -> Self {
        self.pos = Some(pos);
        self
    }

    pub fn with_orbit(mut self, parent: ObjId, distance: f32, angle: Rad) -> Self {
        self.orbit = Some(NewObjOrbit {
            parent,
            distance,
            angle,
        });
        self
    }
}
