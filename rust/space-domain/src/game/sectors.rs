use crate::utils::*;

use space_macros::*;

use serde_json::json;
use std::collections::HashMap;
use crate::game::jsons;
use crate::game::save::{Save, Load};
use crate::game::jsons::JsonValueExtra;

#[derive(Clone,Debug)]
pub struct Jump {
    pub id: JumpId,
    pub sector_id: SectorId,
    pub pos: Position,
    pub to_sector_id: SectorId,
    pub to_pos: Position,
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct SectorId(pub u32);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct JumpId(pub u32);

impl SectorId {
    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug)]
pub struct Sector {
    pub id: SectorId,
}

pub struct Sectors {
    sectors: HashMap<SectorId, Sector>,
    jumps: HashMap<JumpId, Jump>,
    jumps_by_sector: HashMap<SectorId, Vec<JumpId>>,
}

impl Sectors {
    pub fn new() -> Self {
        Sectors {
            sectors: HashMap::new(),
            jumps: Default::default(),
            jumps_by_sector: Default::default()
        }
    }

    pub fn add_sector(&mut self, sector: Sector) {
        info!("sectors", "adding sector {:?}", sector);
        assert!(!self.sectors.contains_key(&sector.id));
        self.sectors.insert(sector.id, sector);
    }

    pub fn add_jump(&mut self, jump: Jump) {
        info!("sectors", "adding jump {:?}", jump);
        assert!(!self.jumps.contains_key(&jump.id));

        self.jumps_by_sector
            .entry(jump.sector_id.clone())
            .and_modify(|list| list.push(jump.id))
            .or_insert(vec![jump.id]);

        self.jumps.insert(jump.id, jump);
    }

    pub fn get(&self, sector_id: &SectorId) -> &Sector {
        self.sectors.get(sector_id).unwrap()
    }

    pub fn list<'a>(&self) -> Vec<SectorId> {
        self.sectors.keys()
            .into_iter()
            .map(|i| *i)
            .collect()
    }

    pub fn find_jump(&self, from: SectorId, to: SectorId) -> Option<&Jump> {
        self.get_jumps(from)
            .into_iter()
            .find(|jump| jump.to_sector_id == to)
    }

    pub fn get_jump(&self, jump_id: JumpId) -> Option<&Jump> {
        self.jumps.get(&jump_id)
    }

    pub fn list_jumps(&self) -> Vec<&Jump> {
        self.jumps.values().into_iter().collect()
    }

    pub fn get_jumps(&self, sector_id: SectorId) -> Vec<&Jump> {
        self.jumps_by_sector.get(&sector_id)
            .map(|jumps| {
                jumps.iter()
                    .flat_map(|jump_id| self.jumps.get(jump_id))
                    .collect()
            })
            .unwrap_or(vec![])
    }

    pub fn save(&self, save: &mut impl Save) {
//        for (sector_id,sector) in self.index.iter() {
//            let jumps: Vec<serde_json::Value> = sector.jumps.iter().map(|jump| {
//                json!({
//                    "to_sector_id": jump.to.0,
//                    "pos": jsons::from_v2(&jump.pos)
//                })
//            }).collect();
//
//            save.add(sector_id.0, "sector", json!({
//                "jumps": jumps
//            }));
//        }
    }

    pub fn load(&mut self, load: &mut impl Load) {
//        for (id, value) in load.get_components("sector") {
//            let mut jumps = vec![];
//
//            for i in value["jumps"].as_array().unwrap().iter() {
//                let to_sector_id = SectorId(i["to_sector_id"].as_i64().unwrap() as u32);
//                let pos = i["pos"].to_v2();
//
//                jumps.push(NewJump { to_sector_id, pos});
//            }
//
//            let ns = NewSector {
//                id: SectorId(*id),
//                jumps: jumps
//            };
//
//            self.add_sector(ns);
//        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sectors() {

    }
}
