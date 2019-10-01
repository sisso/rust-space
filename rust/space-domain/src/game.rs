use crate::utils::*;

use self::actions::*;
use self::commands::*;
use self::docking::Docking;
use self::extractables::Extractables;
use self::locations::Locations;
use self::navigation::Navigations;
use self::new_obj::NewObj;
use self::objects::*;
use self::save::{CanLoad, CanSave, Load, Save};
use self::sectors::*;
use self::wares::*;
use crate::game::events::{Events, ObjEvent, EventKind};

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
pub mod new_obj;
pub mod jsons;
pub mod ship;
pub mod factory;
pub mod events;

pub struct Tick {
    total_time: Seconds,
    delta_time: Seconds
}

pub struct Game {
    pub commands: Commands,
    pub actions: Actions,
    pub sectors: Sectors,
    pub objects: ObjRepo,
    pub locations: Locations,
    pub extractables: Extractables,
    pub cargos: Cargos,
    pub navigations: Navigations,
    pub docking: Docking,
    pub events: Events,
}

impl Game {
    pub fn new() -> Self {
        Game {
            commands: Commands::new(),
            actions: Actions::new(),
            sectors: Sectors::new(),
            objects: ObjRepo::new(),
            locations: Locations::new(),
            extractables: Extractables::new(),
            cargos: Cargos::new(),
            navigations: Navigations::new(),
            docking: Docking::new(),
            events: Events::new(),
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
            let cargo = Cargo::new(new_obj.cargo_size);
            self.cargos.init(&id, cargo);
        }

        self.events.add_obj_event(ObjEvent::new(id, EventKind::Add));

        id
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        self.commands.set_command(obj_id, command);
    }

    pub fn tick(&mut self, total_time: Seconds, delta_time: Seconds) {
        info!("game", &format!("tick delta {} total {}", delta_time.0, total_time.0));
        let tick = Tick { total_time, delta_time };
        self.commands.execute(&tick, &self.objects, &self.extractables, &mut self.actions, &self.locations, &self.sectors, &mut self.cargos);
        self.actions.execute(&tick, &self.sectors, &mut self.locations, &self.extractables, &mut self.cargos, &mut self.events);
    }

    pub fn save(&self, save: &mut impl Save) {
        self.sectors.save(save);
        self.objects.save(save);
        self.locations.save(save);
        self.extractables.save(save);
        self.cargos.save(save);
        self.actions.save(save);
        self.commands.save(save);
    }

    pub fn load(&mut self, load: &mut impl Load) {
        self.sectors.load(load);
        self.objects.load(load);
        self.locations.load(load);
        self.extractables.load(load);
        self.cargos.load(load);
        self.commands.load(load);
        self.actions.load(load);
    }
}