use crate::game::objects::ObjId;
use bevy_ecs::prelude::Event;
use bevy_ecs::system::Resource;

#[derive(Debug, Clone, Copy)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
    Deorbit,
    Orbit,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct GEvent {
    pub id: ObjId,
    pub kind: EventKind,
}

impl GEvent {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        GEvent { id, kind }
    }
}

#[derive(Resource, Debug)]
pub struct GEvents {
    queue: Vec<GEvent>,
}

impl Default for GEvents {
    fn default() -> Self {
        GEvents {
            queue: Default::default(),
        }
    }
}

impl GEvents {
    pub fn push(&mut self, event: GEvent) {
        self.queue.push(event);
    }

    pub fn take(&mut self) -> Vec<GEvent> {
        std::mem::replace(&mut self.queue, vec![])
    }
}
