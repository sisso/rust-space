use crate::game::objects::ObjId;
use crate::game::save::MapEntity;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Event, World};
use bevy_ecs::system::{Command, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
    Deorbit,
    Orbit,
}

#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub struct GEvent {
    pub id: ObjId,
    pub kind: EventKind,
}

impl GEvent {
    pub fn new(id: ObjId, kind: EventKind) -> Self {
        GEvent { id, kind }
    }
}

impl MapEntity for GEvent {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.id.map_entity(entity_map);
    }
}

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
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

    pub fn list(&self) -> &Vec<GEvent> {
        &self.queue
    }
}

impl MapEntity for GEvents {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.queue.map_entity(entity_map);
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
