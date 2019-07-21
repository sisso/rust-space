pub mod sectors;
pub mod objects;
pub mod wares;
pub mod command;

use self::sectors::*;
use self::objects::*;
use self::wares::*;
use self::command::*;

pub struct Game {
    sectors: SectorRepo,
    objects: ObjRepo,
}

impl Game {
    pub fn new() -> Self {
        Game {
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
}
