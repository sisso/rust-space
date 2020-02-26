use specs::prelude::*;
use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;
use crate::game::{GameInitContext, RequireInitializer};

#[derive(Debug, Clone)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
}

#[derive(Debug, Clone, Component)]
pub struct Event {
    pub id: ObjId,
    pub kind: EventKind,
}

impl Event {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        Event { id, kind }
    }
}

pub struct Events;

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
    type SystemData = (Entities<'a>, WriteStorage<'a, Event>);

    fn run(&mut self, (entities, events): Self::SystemData) {
        trace!("running");

        for (e, event) in (&*entities, &events).join() {
            trace!("{:?} removing {:?}", e, event);
            entities.delete(e).unwrap();
        }
    }
}