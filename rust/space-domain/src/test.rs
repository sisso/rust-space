use crate::game::events::GEvents;
use crate::game::utils::{DeltaTime, TotalTime, MIN_DISTANCE, V2};
use bevy_ecs::prelude::*;

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

pub struct TestSystemRunner {
    pub world: World,
    pub scheduler: Schedule,
}

impl TestSystemRunner {
    pub fn new<SystemsType>(systems: impl IntoSystemConfigs<SystemsType>) -> TestSystemRunner {
        let mut world = World::new();
        let mut scheduler = Schedule::default();
        world.insert_resource(GEvents::default());
        scheduler.add_systems(systems);
        TestSystemRunner { world, scheduler }
    }

    pub fn tick(&mut self) {
        self.scheduler.run(&mut self.world);
    }

    pub fn tick_timed(&mut self, delta_time: DeltaTime) {
        let total_time = self
            .world
            .get_resource_mut::<TotalTime>()
            .map(|value| *value)
            .unwrap_or_default();

        self.world.insert_resource(total_time.add(delta_time));
        self.world.insert_resource(delta_time);
        self.scheduler.run(&mut self.world);
    }
}

pub fn test_system<'a, SystemsType, Callback, ReturnType>(
    systems: impl IntoSystemConfigs<SystemsType>,
    add_entities: Callback,
) -> (World, ReturnType)
where
    Callback: FnOnce(&mut World) -> ReturnType,
{
    let mut runner = TestSystemRunner::new(systems);
    let result = add_entities(&mut runner.world);
    runner.tick();
    (runner.world, result)
}
