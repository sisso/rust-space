
use crate::utils::{MIN_DISTANCE, V2};
use specs::prelude::*;

pub fn test_system<'a, SystemType, Callback, ReturnType>(
    system: SystemType,
    add_entities: Callback,
) -> (World, ReturnType)
where
    SystemType: for<'c> System<'c> + Send + 'a,
    Callback: FnOnce(&mut World) -> ReturnType,
{
    let mut world = World::new();

    // create dispatcher for testing
    let mut dispatcher = DispatcherBuilder::new().with(system, "test", &[]).build();

    dispatcher.setup(&mut world);

    let result = add_entities(&mut world);

    dispatcher.dispatch(&world);
    world.maintain();

    (world, result)
}

pub fn assert_v2(value: V2, expected: V2) {
    let distance = value.sub(&expected).length();
    if distance > MIN_DISTANCE {
        panic!(
            "fail, receives {:?} but expect {:?}, distance of {:?}",
            value, expected, distance
        );
    }
}
