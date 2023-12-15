use crate::game::objects::ObjId;
use bevy_ecs::prelude::{Event, World};
use bevy_ecs::system::{Command, Resource};

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

pub struct CommandSendEvent {
    pub event: GEvent,
}

impl Command for CommandSendEvent {
    fn apply(self, world: &mut World) {
        world
            .get_resource_mut::<GEvents>()
            .expect("events not found in resources")
            .push(self.event);
    }
}

impl From<GEvent> for CommandSendEvent {
    fn from(event: GEvent) -> Self {
        CommandSendEvent { event }
    }
}
