use specs::prelude::*;
use crate::utils::{V2, MIN_DISTANCE};

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

pub fn assert_v2(value: V2, expected: V2) {
    assert!(value.sub(&expected).length() < MIN_DISTANCE);
}