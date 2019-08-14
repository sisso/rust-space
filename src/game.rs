pub mod sectors;
pub mod objects;
pub mod wares;
pub mod actions;
pub mod commands;
pub mod locations;
pub mod template;
pub mod extractables;
pub mod navigation;
pub mod docking;
pub mod save;

use crate::utils::*;

use self::sectors::*;
use self::objects::*;
use self::wares::*;
use self::commands::*;
use self::actions::*;
use crate::game::locations::{Locations, Location};
use crate::game::extractables::{Extractables, Extractable};
use crate::game::navigation::Navigations;
use crate::game::docking::Docking;
use crate::game::save::Save;

pub struct Tick {
    total_time: Seconds,
    delta_time: Seconds
}

pub struct Game {
    commands: Commands,
    actions: Actions,
    sectors: SectorRepo,
    objects: ObjRepo,
    locations: Locations,
    extractables: Extractables,
    cargos: Cargos,
    navigations: Navigations,
    docking: Docking,
}

impl Game {
    pub fn new() -> Self {
        Game {
            commands: Commands::new(),
            actions: Actions::new(),
            sectors: SectorRepo::new(),
            objects: ObjRepo::new(),
            locations: Locations::new(),
            extractables: Extractables::new(),
            cargos: Cargos::new(),
            navigations: Navigations::new(),
            docking: Docking::new(),
        }
    }

    pub fn add_sector(&mut self, sector: NewSector) {
        self.sectors.add_sector(sector);
    }

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        let id = self.objects.create(new_obj.has_dock);

        self.locations.init(&id);

        if new_obj.ai {
            self.commands.init(id);
            self.actions.init(id);
        }

        new_obj.location.iter().for_each(|location| {
            self.locations.set_location(&id, location.clone());
        });

        new_obj.speed.iter().for_each(|speed| {
            self.locations.set_moveable(&id, speed.clone());
            self.navigations.init(&id);
        });

        new_obj.extractable.iter().for_each(|i| {
            self.extractables.set_extractable(&id, i.clone());
        });

        if new_obj.cargo_size > 0.0 {
            self.cargos.init(&id, new_obj.cargo_size);
        }

        id
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        self.commands.set_command(obj_id, command);
    }

    pub fn tick(&mut self, total_time: Seconds, delta_time: Seconds) {
        Log::info("game", &format!("tick delta {} total {}", delta_time.0, total_time.0));
        let tick = Tick { total_time, delta_time };
        self.commands.execute(&tick, &self.objects, &self.extractables, &mut self.actions, &self.locations, &self.sectors, &mut self.cargos);
        self.actions.execute(&tick, &self.sectors, &mut self.locations, &self.extractables, &mut self.cargos);
    }

    pub fn save(&self, save: &mut impl Save) {
        self.commands.save(save);
        self.actions.save(save);
        self.sectors.save(save);
        self.objects.save(save);
        self.locations.save(save);
        self.extractables.save(save);
        self.cargos.save(save);
    }
}


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
            ai: false
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
        self.location = Some(Location::Space {
            sector_id,
            pos
        });
        self
    }

    pub fn at_dock(mut self, obj_id: ObjId) -> Self {
        self.location = Some(Location::Docked { obj_id });
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
