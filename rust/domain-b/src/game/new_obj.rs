use bevy_ecs::prelude::*;

use crate::game::building_site::BuildingSite;
use crate::game::production_cost::ProductionCost;
use commons::math::{Rad, P2, P2I};

use crate::game::commands::Command;
use crate::game::extractables::Extractable;
use crate::game::factory::Factory;
use crate::game::locations::*;
use crate::game::objects::ObjId;
use crate::game::sectors::*;
use crate::game::shipyard::Shipyard;
use crate::game::wares::{Cargo, Volume, WareAmount};
use crate::game::work::WorkUnit;
use crate::utils::*;

#[derive(Debug, Clone, Component, Default)]
pub struct NewObj {
    pub speed: Option<Speed>,
    pub cargo: Option<Cargo>,
    pub extractable: Option<Extractable>,
    pub location_space: Option<LocationSpace>,
    pub location_docked: Option<LocationDocked>,
    pub can_dock: bool,
    pub fleet: bool,
    pub docking: bool,
    pub station: bool,
    pub sector: Option<P2I>,
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
    pub location_orbit: Option<LocationOrbit>,
    pub building_site: Option<BuildingSite>,
    pub production_cost: Option<ProductionCost>,
}

impl NewObj {
    pub fn new() -> NewObj {
        Default::default()
    }

    pub fn with_cargo_size(mut self, cargo_size: Volume) -> Self {
        self.cargo = Some(Cargo::new(cargo_size));
        self
    }

    pub fn with_cargo(mut self, cargo: Cargo) -> Self {
        self.cargo = Some(cargo);
        self
    }

    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn at_position(mut self, sector_id: SectorId, pos: P2) -> Self {
        self.location_space = Some(LocationSpace { pos, sector_id });
        self
    }

    pub fn at_dock(mut self, parent_id: ObjId) -> Self {
        self.location_docked = Some(LocationDocked { parent_id });
        self
    }

    pub fn extractable(mut self, extractable: Extractable) -> Self {
        self.extractable = Some(extractable);
        self
    }

    pub fn with_docking(mut self) -> Self {
        self.docking = true;
        self
    }

    pub fn can_dock(mut self) -> Self {
        self.can_dock = true;
        self
    }

    pub fn with_station(mut self) -> Self {
        self.station = true;
        self
    }

    pub fn with_building_site(mut self, bs: BuildingSite) -> Self {
        self.building_site = Some(bs);
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

    pub fn with_sector(mut self, pos: P2I) -> Self {
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

    pub fn with_code<IntoString: Into<String>>(mut self, code: IntoString) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_label<IntoString: Into<String>>(mut self, label: IntoString) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_pos(mut self, pos: V2) -> Self {
        self.pos = Some(pos);
        self
    }

    pub fn with_orbit(
        mut self,
        parent_id: ObjId,
        distance: f32,
        start_angle: Rad,
        speed: Speed,
        start_time: TotalTime,
    ) -> Self {
        self.location_orbit = Some(LocationOrbit {
            parent_id,
            distance,
            start_time,
            speed,
            start_angle,
        });
        self
    }

    pub fn with_production_cost(mut self, work: WorkUnit, cost: Vec<WareAmount>) -> Self {
        self.production_cost = Some(ProductionCost { work, cost });
        self
    }
}
