use specs::prelude::*;
use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;

#[derive(Debug, Clone)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
}

#[derive(Debug, Clone)]
pub struct ObjEvent {
    id: ObjId,
    kind: EventKind,
}

impl ObjEvent {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        ObjEvent { id, kind }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Events {
   list: Vec<ObjEvent>,
}

/// Events are added in normal dispatcher and read later_dispatcher, then clean up in clean
/// dispatcher
impl Events {
    pub fn new(list: Vec<ObjEvent>) -> Self {
        Events { list }
    }

    pub fn single(event: ObjEvent) -> Self {
        Events { list: vec![event] }
    }

    pub fn init_world_late(world: &mut World, late_dispatcher: &mut DispatcherBuilder) {
        late_dispatcher.add(
            ClearEventsSystem,
            "clear_events_system",
            &[],
        );
    }
}

pub struct ClearEventsSystem;

impl<'a> System<'a> for ClearEventsSystem {
    type SystemData = (Entities<'a>, WriteStorage<'a, Events>);

    fn run(&mut self, (entities, events): Self::SystemData) {
        trace!("running");

        for (e, _event) in (&*entities, &events).join() {
            trace!("{:?} removing", e);
            entities.delete(e);
        }
    }
}