use bevy_ecs::prelude::*;
use bevy_ecs::system::CommandQueue;

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
