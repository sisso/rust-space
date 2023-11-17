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
    let mut runner = TestSystemRunner::new(system);
    let result = add_entities(&mut runner.world);
    runner.tick();
    (runner.world, result)
}

pub fn assert_v2(value: V2, expected: V2) {
    let distance = (value - expected).length();
    if distance > MIN_DISTANCE {
        panic!(
            "fail, receives {:?} but expect {:?}, distance of {:?}",
            value, expected, distance
        );
    }
}

pub fn init_trace_log() {
    _ = env_logger::builder()
        .filter(None, log::LevelFilter::Trace)
        .try_init();
}

pub struct TestSystemRunner<'a> {
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> TestSystemRunner<'a> {
    pub fn new<SystemType>(system: SystemType) -> TestSystemRunner<'a>
    where
        SystemType: for<'c> System<'c> + Send + 'a,
    {
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new().with(system, "test", &[]).build();
        dispatcher.setup(&mut world);
        TestSystemRunner { world, dispatcher }
    }

    pub fn tick(&mut self) {
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();
    }
}
