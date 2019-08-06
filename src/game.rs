pub mod sectors;
pub mod objects;
pub mod wares;
pub mod action;
pub mod commands;
pub mod locations;
pub mod template;
pub mod extractables;

use crate::utils::*;

use self::sectors::*;
use self::objects::*;
use self::wares::*;
use self::commands::*;
use self::action::*;
use crate::game::locations::Locations;
use crate::game::extractables::Extractables;

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
    extractables: Extractables
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
        }
    }

    pub fn add_sector(&mut self, sector: NewSector) {
        self.sectors.add_sector(sector);
    }

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        let id = self.objects.add_object(&new_obj);
        self.locations.init(&id);

        if new_obj.ai {
            self.commands.init(id);
            self.actions.init(id);
        }

        new_obj.location.iter().for_each(|location| {
            self.locations.set_location(&id, location.clone());
        });

        new_obj.extractable.iter().for_each(|i| {
            self.extractables.set_extractable(&id, i.clone());
        });

        id
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        self.commands.set_command(obj_id, command);
    }

    pub fn tick(&mut self, total_time: Seconds, delta_time: Seconds) {
        Log::info("game", &format!("tick {}/{}", delta_time.0, total_time.0));
        let tick = Tick { total_time, delta_time };
        self.commands.tick(&tick, &self.extractables,&mut self.actions, &mut self.locations, &self.sectors);
        self.actions.tick(&tick, &mut self.objects, &self.sectors, &mut self.locations);
    }
}
