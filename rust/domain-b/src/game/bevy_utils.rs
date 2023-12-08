use crate::game::events::{GEvent, GEvents};
use bevy_ecs::prelude::*;
use bevy_ecs::system::{Command, CommandQueue, SystemParam, SystemState};
use space_galaxy::system_generator::GenerateError::Generic;

pub trait WorldExt {
    fn run_commands<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(Commands) -> T;
}

impl WorldExt for World {
    fn run_commands<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(Commands) -> T,
    {
        let mut command_queue = CommandQueue::default();
        let commands = Commands::new(&mut command_queue, self);
        let result = f(commands);
        command_queue.apply(self);
        result
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
