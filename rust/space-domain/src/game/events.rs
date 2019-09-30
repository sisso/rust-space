use crate::game::objects::ObjId;

#[derive(Debug, Clone, Copy)]
pub enum EventKind {
    Add,
    Move,
    Jump
}

#[derive(Debug, Clone)]
pub struct ObjEvent {
    pub id: ObjId,
    pub kind: EventKind,
}

impl ObjEvent {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        ObjEvent { id, kind }
    }
}

pub struct Events {
    obj_events: Vec<ObjEvent>
}

impl Events {
    pub fn new() -> Self {
        Events {
            obj_events: vec![]
        }
    }

    pub fn add_obj_event(&mut self, e: ObjEvent) {
        self.obj_events.push(e);
    }

    pub fn take(&mut self) -> Vec<ObjEvent> {
        std::mem::replace(&mut self.obj_events, vec![])
    }
}
