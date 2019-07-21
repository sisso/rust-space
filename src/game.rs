pub mod ai;
pub mod sectors;
pub mod objects;
pub mod wares;
pub mod action;
pub mod command;

use self::sectors::*;
use self::objects::*;
use self::wares::*;
use self::ai::*;
use self::command::*;
use crate::utils::*;
use crate::game::action::{Actions, Action};

pub struct Tick {
    total_time: Seconds,
    delta_time: Seconds
}

pub struct Game {
    ai: Ai,
    actions: Actions,
    sectors: SectorRepo,
    objects: ObjRepo,
}

impl Game {
    pub fn new() -> Self {
        Game {
            ai: Ai::new(),
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
        let tick = Tick { total_time, delta_time };
        self.ai.tick(&tick, &mut self.objects, & self.sectors);
    }
}
