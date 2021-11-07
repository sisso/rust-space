use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;
use crate::game::{GameInitContext, RequireInitializer};
use specs::prelude::*;
use std::borrow::BorrowMut;

#[derive(Debug, Clone)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: ObjId,
    pub kind: EventKind,
}

impl Event {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        Event { id, kind }
    }
}

pub struct Events {
    queue: Vec<Event>,
}

impl Default for Events {
    fn default() -> Self {
        Events {
            queue: Default::default(),
        }
    }
}

impl Events {
    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn take(&mut self) -> Vec<Event> {
        std::mem::replace(&mut self.queue, vec![])
    }
}
