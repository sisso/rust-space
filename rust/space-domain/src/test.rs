use specs::prelude::*;

pub fn test_system<'a, T, F, J>(system: T, add_entities: F) -> (World, J) where
    T: for<'c> System<'c> + Send + 'a,
    F: FnOnce(&mut World) -> J
{
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(system, "test", &[])
        .build();
    dispatcher.setup(&mut world);
    let result = add_entities(&mut world);
    dispatcher.dispatch(&world);
    world.maintain();
    (world, result)
}
