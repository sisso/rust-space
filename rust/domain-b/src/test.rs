use crate::game::utils::{MIN_DISTANCE, V2};
use bevy_ecs::prelude::*;
use log::SetLoggerError;

#[deprecated]
pub fn assert_v2(value: V2, expected: V2) {
    let distance = (value - expected).length();
    if distance > MIN_DISTANCE {
        panic!(
            "fail, receives {:?} but expect {:?}, distance of {:?}",
            value, expected, distance
        );
    }
}

pub fn init_trace_log() -> Result<(), SetLoggerError> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Trace)
        .try_init()
}
