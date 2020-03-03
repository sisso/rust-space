use specs::prelude::*;
use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;
use crate::game::{GameInitContext, RequireInitializer};
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
            queue: Default::default()
        }
    }
}

impl Events {
    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &'a Event> + 'a {
        self.queue.iter()
    }
}

impl RequireInitializer for Events {
    fn init(context: &mut GameInitContext) {
        context.cleanup_dispatcher.add(
            ClearEventsSystem,
            "clear_events_system",
            &[],
        );
    }
}

/// Remove all entities with events
pub struct ClearEventsSystem;

impl<'a> System<'a> for ClearEventsSystem {
    type SystemData = (Write<'a, Events>);

    fn run(&mut self, (mut events): Self::SystemData) {
        trace!("running");
        events.borrow_mut().clear();
    }
}