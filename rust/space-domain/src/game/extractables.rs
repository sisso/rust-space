use super::objects::{ObjId};
use crate::utils::*;

use std::collections::HashMap;
use crate::game::wares::WareId;
use crate::game::save::{Save, Load};
use crate::game::jsons::*;


#[derive(Clone,Debug)]
pub struct Extractable {
    pub ware_id: WareId,
    pub time: Seconds,
}


#[derive(Clone, Debug)]
struct State {
    extractable: Extractable
}

pub struct Extractables {
    index: HashMap<ObjId, State>,
}

impl Extractables {
    pub fn new() -> Self {
        Extractables {
            index: HashMap::new()
        }
    }

    pub fn init(&mut self, id: &ObjId, extractable: Extractable) {
        self.set_extractable(id, extractable);
    }

    pub fn set_extractable(&mut self, obj_id: &ObjId, extractable: Extractable) {
        info!("extractable", &format!("set {:?}: {:?}", obj_id, extractable));

        if self.index.contains_key(obj_id) {
            let mut state = self.index.get_mut(&obj_id).unwrap();
            state.extractable = extractable;
        } else {
            let state = State { extractable };

            self.index.insert(*obj_id, state);
        }
    }

    pub fn get_extractable(&self, id: &ObjId) -> &Extractable {
        let state = self.index.get(id).unwrap();
        &state.extractable
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn save(&self, save: &mut impl Save) {
        use serde_json::json;

        for (k,v) in self.index.iter() {
            let ware_id = v.extractable.ware_id.0;
            let seconds = v.extractable.time.0;

            save.add(k.0, "extractable", json!({
                "ware_id": ware_id,
                "seconds": seconds,
            }));
        }
    }

    pub fn load(&mut self, load: &mut impl Load) {
        for (id, value) in load.get_components("extractable") {
            let extractable = Extractable {
                ware_id: WareId(value["ware_id"].to_u32()),
                time: Seconds(value["seconds"].to_f32()),
            };

            self.set_extractable(&ObjId(*id), extractable);
        }
    }
}
