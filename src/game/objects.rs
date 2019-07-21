use crate::utils::*;
use super::wares::*;
use super::command::*;

use std::collections::HashMap;
use crate::game::action::Action;
use crate::game::sectors::SectorId;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct ObjId(pub u32);

#[derive(Clone,Copy,Debug)]
pub struct Speed(pub f32);

pub struct NewObj {
    pub speed: Option<Speed>,
    pub cargo_size: u32,
    pub extractable: Option<Extractable>,
    pub location: Option<Location>,
    pub can_dock: bool,
    pub has_dock: bool,
}

impl NewObj {
    pub fn new() -> NewObj {
        NewObj {
            speed: None,
            cargo_size: 0,
            extractable: None,
            location: None,
            can_dock: false,
            has_dock: false
        }
    }

    pub fn with_cargo(mut self, cargo: u32) -> Self {
        self.cargo_size = cargo;
        self
    }

    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn at_position(mut self, sector_id: SectorId, pos: Position) -> Self {
        self.location = Some(Location::Space {
            sector_id,
            pos
        });
        self
    }

    pub fn at_dock(mut self, obj_id: ObjId) -> Self {
        self.location = Some(Location::Docked { obj_id });
        self
    }

    pub fn extractable(mut self, extractable: Extractable) -> Self {
        self.extractable = Some(extractable);
        self
    }

    pub fn has_dock(mut self) -> Self {
        self.has_dock = true;
        self
    }

    pub fn can_dock(mut self) -> Self {
        self.can_dock = true;
        self
    }
}

#[derive(Clone, Debug)]
pub enum Location {
    Docked { obj_id: ObjId },
    Space { sector_id: SectorId, pos: Position },
}

#[derive(Debug, Clone)]
pub struct Obj {
    pub id: ObjId,
    pub max_speed: Option<Speed>,
    pub cargo: Cargo,
    pub location: Location,
    pub command: Command,
    pub can_dock: bool,
    pub has_dock: bool,
    pub action: Action,
    pub action_delay: Option<Seconds>,
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

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        let id = ObjId(self.ids.next());

        let obj = Obj {
            id: id,
            max_speed: new_obj.speed,
            cargo: Cargo::new(new_obj.cargo_size),
            location: new_obj.location.unwrap(),
            command: Command::Idle,
            can_dock: new_obj.can_dock,
            has_dock: new_obj.has_dock,
            action: Action::Idle,
            action_delay: None,
        };

        Log::info("objects", &format!("adding object {:?}", obj));

        if self.index.insert(obj.id, obj).is_some() {
            panic!("can not add already existent obj")
        }
        id
    }

    pub fn set_command(&mut self, obj_id: ObjId, command: Command) {
        let mut obj = self.index.get_mut(&obj_id).unwrap();
        Log::info("objects", &format!("set command {:?}: {:?}", obj.id, command));
        obj.command = command;
    }

    pub fn set_action(&mut self, obj_id: ObjId, action: Action) {
        let mut obj = self.index.get_mut(&obj_id).unwrap();
        Log::info("objects", &format!("set action {:?}: {:?}", obj.id, action));
        obj.action = action;
    }

    pub fn set_location(&mut self, obj_id: ObjId, location: Location) {
        let mut obj = self.index.get_mut(&obj_id).unwrap();
        Log::info("objects", &format!("set location {:?}: {:?}", obj.id, location));
        obj.location = location;
    }

    pub fn get(&self, obj_id: &ObjId) -> &Obj {
        self.index.get(obj_id).unwrap()
    }

//    pub fn _update(&mut self, obj: Obj) {
//        Log::info("objects", &format!("update object {:?}", obj));
//
//        if self.index.insert(obj.id, obj).is_none() {
//            panic!("can not update non existent object");
//        }
//    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Obj> + 'a {
        self.index.values()
    }
}
