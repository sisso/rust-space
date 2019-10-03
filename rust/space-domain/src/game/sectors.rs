use crate::utils::*;

use space_macros::*;

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

#[derive(Clone,Debug)]
pub struct Jump2Sector {
    pub sector_id: SectorId,
    pub pos: V2,
}

#[derive(Clone,Debug)]
pub struct Jump2 {
    pub a: Jump2Sector,
    pub b: Jump2Sector,
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct SectorId(pub u32);

impl SectorId {
    pub fn value(&self) -> u32 {
        self.0
    }
}

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

        info!("sectors", "adding {:?}", sector);

        self.index.insert(sector.id, sector);
    }

    pub fn get(&self, sector_id: &SectorId) -> &Sector {
        self.index.get(sector_id).unwrap()
    }

    pub fn list(&self) -> Vec<SectorId> {
        self.index.keys()
            .into_iter()
            .map(|i| *i)
            .collect()
    }

    // TODO: support more that one jump
    pub fn find_jump_at(&self, sector_id: &SectorId, jump_position: &Position) -> Option<&Jump> {
        let sector = self.get(sector_id);
        sector.jumps.get(0)
    }

    pub fn find_target_jump(&self, sector_id: SectorId, target_sector_id: SectorId) -> Jump {
        self.get(&sector_id).jumps.iter()
            .find(|jump| jump.to == target_sector_id)
            .map(|jump| jump.clone())
            .unwrap()
    }

    pub fn get_jumps(&self) -> Vec<Jump2> {
        vec![]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sectors_jumps() {
        let mut sectors = Sectors::new();

        sectors.add_sector(NewSector {
            id: SectorId(0),
            jumps: vec![
                NewJump {
                    to_sector_id: SectorId(1),
                    pos: V2::new(1.0, 0.0)
                }
            ]
        });

        sectors.add_sector(NewSector {
            id: SectorId(1),
            jumps: vec![
                NewJump {
                    to_sector_id: SectorId(0),
                    pos: V2::new(0.0, 1.0)
                }
            ]
        });

        let jumps = sectors.get_jumps();

        assert_eq!(jumps.len(), 1);

        let jump = jumps.get(0).unwrap();
        assert_eq!(jump.a.sector_id, SectorId(0));
        assert_eq!(jump.a.pos, V2::new(1.0, 0.0));

        assert_eq!(jump.b.sector_id, SectorId(1));
        assert_eq!(jump.b.pos, V2::new(0.0, 1.0));
    }
}
