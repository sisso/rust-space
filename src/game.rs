pub mod sectors;
pub mod objects;
pub mod wares;
pub mod action;
pub mod commands;

use crate::utils::*;

use self::sectors::*;
use self::objects::*;
use self::wares::*;
use self::commands::*;
use self::action::*;

pub struct Tick {
    total_time: Seconds,
    delta_time: Seconds
}

pub struct Game {
    commands: Commands,
    actions: Actions,
    sectors: SectorRepo,
    objects: ObjRepo,
}

impl Game {
    pub fn new() -> Self {
        Game {
            commands: Commands::new(),
            actions: Actions::new(),
            sectors: SectorRepo::new(),
            objects: ObjRepo::new(),
        }
    }

    pub fn add_sector(&mut self, sector: NewSector) {
        self.sectors.add_sector(sector);
    }

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        self.objects.add_object(new_obj)
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        self.objects.set_command(obj_id, command);
    }

    pub fn tick(&mut self, total_time: Seconds, delta_time: Seconds) {
        Log::info("game", &format!("tick {}/{}", delta_time.0, total_time.0));
        let tick = Tick { total_time, delta_time };
        self.commands.tick(&tick, &mut self.objects, & self.sectors);
        self.actions.tick(&tick, &mut self.objects, & self.sectors);
    }
}
