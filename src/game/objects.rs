use std::collections::HashMap;

use crate::utils::*;
use super::wares::*;
use super::commands::*;
use super::actions::Action;
use super::sectors::SectorId;
use crate::game::locations::Location;
use crate::game::extractables::Extractable;
use crate::game::save::Save;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct ObjId(pub u32);

#[derive(Debug, Clone)]
pub struct Obj {
    pub id: ObjId,
    pub has_dock: bool,
}

impl Obj {
}

pub struct ObjRepo {
    ids: NextId,
    index: HashMap<ObjId, Obj>
}

impl ObjRepo {
    pub fn new() -> Self {
        ObjRepo {
            ids: NextId::new(),
            index: HashMap::new()
        }
    }

    pub fn create(&mut self, has_dock: bool) -> ObjId {
        let id = ObjId(self.ids.next());

        let obj = Obj {
            id,
            has_dock,
        };

        Log::info("objects", &format!("adding object {:?}", obj));

        if self.index.insert(obj.id, obj).is_some() {
            panic!("can not add already existent obj")
        }

        id
    }

    pub fn get(&self, obj_id: &ObjId) -> &Obj {
        self.index.get(obj_id).unwrap()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Obj> + 'a {
        self.index.values()
    }

    pub fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (k,v) in self.index.iter() {
            save.add(k.0, "object", json!({
                "has_dock": v.has_dock
            }));
        }
    }
}
