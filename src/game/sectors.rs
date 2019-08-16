use crate::utils::*;

use std::collections::HashMap;
use crate::game::save::Save;

#[derive(Clone,Debug)]
pub struct Jump {
    pub to: SectorId,
    pub pos: V2,
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
    pub id: SectorId,
    pub jumps: Vec<Jump>
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

    pub fn get(&self, sector_id: &SectorId) -> &Sector {
        self.index.get(sector_id).unwrap()
    }

    // TODO: support more that one jump
    pub fn find_jump_at(&self, sector_id: &SectorId, jump_position: &Position) -> Option<&Jump> {
        let sector = self.get(sector_id);
        sector.jumps.get(0)
    }

    pub fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (sector_id,sector) in self.index.iter() {
            let jumps: Vec<serde_json::Value> = sector.jumps.iter().map(|jump| {
                json!({
                    "to_sector_id": jump.to.0,
                    "pos": (jump.pos.x, jump.pos.y)
                })
            }).collect();

            save.add(sector_id.0, "sector", json!({
                "jumps": jumps
            }));
        }
    }
}
