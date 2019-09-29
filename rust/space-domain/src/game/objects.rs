use std::collections::HashMap;

use crate::utils::*;
use super::wares::*;
use super::commands::*;
use super::actions::Action;
use super::sectors::SectorId;
use crate::game::locations::Location;
use crate::game::extractables::Extractable;
use crate::game::save::{Save, Load};

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

        info!("objects", &format!("adding object {:?}", obj));

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

    pub fn load(&mut self, load: &mut impl Load) {
        let mut max_id: u32 = 0;

        for (id, value) in load.get_components("object") {
            max_id = max_id.max(*id);

            let obj = Obj {
                id: ObjId(*id),
                has_dock: value["has_dock"].as_bool().unwrap(),
            };

            self.index.insert(ObjId(*id), obj);
        }

        self.ids = NextId::from(max_id);
    }
}
