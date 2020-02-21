use std::any::Any;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use shred::{Read, ResourceId, SystemData, World, Write};
use specs::prelude::*;
use specs_derive::*;

use crate::game::jsons;
use crate::game::jsons::JsonValueExtra;
use crate::game::save::{Load, Save};
use crate::utils::*;

#[derive(Debug, Clone, Component)]
pub struct SectorsIndex {}

impl Default for SectorsIndex {
    fn default() -> Self {
        SectorsIndex {}
    }
}

impl SectorsIndex {
    pub fn add_jump(&mut self, sector_1: SectorId, sector_2: SectorId) {}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jump {
    pub id: JumpId,
    pub sector_id: SectorId,
    pub pos: Position,
    pub to_sector_id: SectorId,
    pub to_pos: Position,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct SectorId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct JumpId(pub u32);

impl SectorId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sector {
    pub id: SectorId,
}

// TODO: migrate to specs
#[derive(Debug, Clone, Component)]
pub struct Sectors {
    sectors: HashMap<SectorId, Sector>,
    jumps: HashMap<JumpId, Jump>,
    jumps_by_sector: HashMap<SectorId, Vec<JumpId>>,
}

impl Default for Sectors {
    fn default() -> Self {
        Sectors::new()
    }
}

impl Sectors {
    pub fn new() -> Self {
        Sectors {
            sectors: HashMap::new(),
            jumps: Default::default(),
            jumps_by_sector: Default::default(),
        }
    }

    pub fn add_sector(&mut self, sector: Sector) {
        info!("adding sector {:?}", sector);
        assert!(!self.sectors.contains_key(&sector.id));
        self.sectors.insert(sector.id, sector);
    }

    pub fn add_jump(&mut self, jump: Jump) {
        info!("adding jump {:?}", jump);
        assert!(!self.jumps.contains_key(&jump.id));

        self.jumps_by_sector
            .entry(jump.sector_id.clone())
            .and_modify(|list| list.push(jump.id))
            .or_insert(vec![jump.id]);

        self.jumps.insert(jump.id, jump);
    }

    pub fn get(&self, sector_id: SectorId) -> &Sector {
        self.sectors.get(&sector_id).unwrap()
    }

    pub fn list<'a>(&self) -> Vec<SectorId> {
        self.sectors.keys().into_iter().map(|i| *i).collect()
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
        self.jumps_by_sector
            .get(&sector_id)
            .map(|jumps| {
                jumps
                    .iter()
                    .flat_map(|jump_id| self.jumps.get(jump_id))
                    .collect()
            })
            .unwrap_or(vec![])
    }

    pub fn save(&self, save: &mut impl Save) {
        for (id, sector) in &self.sectors {
            save.add(sector.id.0, "sector", json!({}));
        }

        for (id, jump) in &self.jumps {
            save.add(jump.id.0, "jump", serde_json::to_value(jump).unwrap());
        }
    }

    pub fn load(&mut self, load: &mut impl Load) {
        for (id, value) in load.get_components("sector") {
            self.add_sector(Sector { id: SectorId(*id) });
        }

        for (id, value) in load.get_components("jump") {
            let jump: Jump = <Jump as Deserialize>::deserialize(value).unwrap();
            self.add_jump(jump);
        }
    }
}

//#[cfg(test)]
pub mod test_scenery {
    use super::*;

    pub const SECTOR_0: SectorId = SectorId(0);
    pub const SECTOR_1: SectorId = SectorId(1);

    pub const JUMP_0_TO_1: Jump = Jump {
        id: JumpId(0),
        sector_id: SECTOR_0,
        pos: Position { x: 4.0, y: 0.0 },
        to_sector_id: SECTOR_1,
        to_pos: Position { x: 0.0, y: 3.0 },
    };

    pub const JUMP_1_TO_0: Jump = Jump {
        id: JumpId(1),
        sector_id: SECTOR_1,
        pos: Position { x: 0.0, y: 3.0 },
        to_sector_id: SECTOR_0,
        to_pos: Position { x: 4.0, y: 0.0 },
    };

    pub fn new_test_sectors() -> Sectors {
        let mut sectors = Sectors::new();

        sectors.add_sector(Sector { id: SECTOR_0 });
        sectors.add_sector(Sector { id: SECTOR_1 });
        sectors.add_jump(JUMP_0_TO_1.clone());
        sectors.add_jump(JUMP_1_TO_0.clone());

        sectors
    }
}

#[cfg(test)]
mod test {
    use super::test_scenery::*;
    use super::*;
    use crate::game::save::LoadSaveBuffer;

    #[test]
    fn test_sectors() {
        let mut buffer = LoadSaveBuffer::new();

        {
            let mut sectors = Sectors::new();
            sectors.add_sector(Sector { id: SectorId(0) });
            sectors.add_sector(Sector { id: SectorId(1) });
            sectors.add_jump(Jump {
                id: JumpId(0),
                sector_id: SectorId(0),
                pos: V2 { x: 0.0, y: 5.0 },
                to_sector_id: SectorId(1),
                to_pos: V2 { x: 3.1, y: 2.2 },
            });
            sectors.save(&mut buffer);
        }

        println!("{:?}", buffer);

        let mut sector = Sectors::new();
        sector.load(&mut buffer);

        sector.get(SectorId(0));
        sector.get(SectorId(1));
    }
}
