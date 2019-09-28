use crate::utils::*;

use serde_json::json;
use std::collections::HashMap;
use crate::game::jsons;
use crate::game::save::{Save, Load};
use crate::game::jsons::JsonValueExtra;

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

pub struct Sectors {
    index: HashMap<SectorId, Sector>
}

impl Sectors {
    pub fn new() -> Self {
        Sectors {
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
        for (sector_id,sector) in self.index.iter() {
            let jumps: Vec<serde_json::Value> = sector.jumps.iter().map(|jump| {
                json!({
                    "to_sector_id": jump.to.0,
                    "pos": jsons::from_v2(&jump.pos)
                })
            }).collect();

            save.add(sector_id.0, "sector", json!({
                "jumps": jumps
            }));
        }
    }

    pub fn load(&mut self, load: &mut impl Load) {
        for (id, value) in load.get_components("sector") {
            let mut jumps = vec![];

            for i in value["jumps"].as_array().unwrap().iter() {
                let to_sector_id = SectorId(i["to_sector_id"].as_i64().unwrap() as u32);
                let pos = i["pos"].to_v2();

                jumps.push(NewJump { to_sector_id, pos});
            }

            let ns = NewSector {
                id: SectorId(*id),
                jumps: jumps
            };

            self.add_sector(ns);
        }
    }
}
