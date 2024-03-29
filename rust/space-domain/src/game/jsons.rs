use commons::math::V2;
use serde_json::json;
use serde_json::Value;

pub fn from_v2(v: &V2) -> Value {
    json!((v.x, v.y))
}

pub trait JsonValueExtra {
    fn to_f32(&self) -> f32;
    fn to_u32(&self) -> u32;
    fn as_v2(&self) -> Option<V2>;
    fn to_v2(&self) -> V2;
    fn as_opt(&self) -> Option<&Value>;
}

impl JsonValueExtra for Value {
    fn to_f32(&self) -> f32 {
        self.as_f64().unwrap() as f32
    }

    fn to_u32(&self) -> u32 {
        self.as_u64().unwrap() as u32
    }

    fn as_v2(&self) -> Option<V2> {
        match self.as_array() {
            Some(vec) => Some(V2::new(vec[0].to_f32(), vec[1].to_f32())),
            _ => None,
        }
    }

    fn to_v2(&self) -> V2 {
        self.as_v2().unwrap()
    }

    fn as_opt(&self) -> Option<&Value> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }
}
