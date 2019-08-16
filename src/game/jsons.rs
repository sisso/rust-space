use serde_json::Value;
use serde_json::json;

use crate::utils::V2;

pub fn from_v2(v: V2) -> Value {
    json!((v.x, v.y))
}

pub trait JsonValueExtra {
    fn as_f32(&self) -> f32;
    fn as_u32(&self) -> u32;
    fn as_v2(&self) -> V2;
}

impl JsonValueExtra for Value {
    fn as_f32(&self) -> f32 {
        match *self {
            Value::Number(ref n) => n.as_f64().unwrap() as f32,
            _ => panic!(),
        }
    }

    fn as_u32(&self) -> u32 {
        match *self {
            Value::Number(ref n) => n.as_i64().unwrap() as u32,
            _ => panic!(),
        }
    }

    fn as_v2(&self) -> V2 {
        match self {
            Value::Array(vec) => V2::new(vec[0].as_f32(), vec[1].as_f32()),
            _ => panic!(),
        }
    }
}
