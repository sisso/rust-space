use crate::utils::*;
use crate::log::*;

use std::collections::HashMap;

#[derive(Clone,Debug)]
pub struct Jump {
    to: SectorId,
    pos: V2,
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct SectorId(pub u32);

pub struct NewJump {
    pub to_sector_id: SectorId,
    pub pos: Position,
}

pub struct NewSector {
    pub id: SectorId,
    pub jumps: Vec<NewJump>
}

#[derive(Debug)]
pub struct Sector {
    id: SectorId,
    jumps: Vec<Jump>
}

pub struct SectorRepo {
    index: HashMap<SectorId, Sector>
}

impl SectorRepo {
    pub fn new() -> Self {
        SectorRepo {
            index: HashMap::new()
        }
    }

    pub fn add_sector(&mut self, sector: NewSector) {
        let sector = Sector {
            id: sector.id,
            jumps: sector.jumps.into_iter().map(|i| {
                Jump {
                    to: i.to_sector_id,
                    pos: i.pos,
                }
            }).collect()
        };

        Log::info("sectors", &format!("adding {:?}", sector));

        self.index.insert(sector.id, sector);
    }
}
