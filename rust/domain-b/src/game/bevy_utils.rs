use bevy_ecs::prelude::*;
use bevy_ecs::system::CommandQueue;

pub fn run_commands<T, F>(world: &mut World, f: F) -> T
where
    F: FnOnce(&mut Commands) -> T,
{
    let mut command_queue = CommandQueue::default();
    let mut commands = Commands::new(&mut command_queue, world);
    let result = f(&mut commands);
    command_queue.apply(world);
    result
}
