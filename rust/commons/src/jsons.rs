use serde_json::Value;

pub trait JsonValueExtra {
    fn to_f32(&self) -> f32;
    fn to_u32(&self) -> u32;
    fn as_opt(&self) -> Option<&Value>;
    fn strip_nulls(&mut self);
}

impl JsonValueExtra for Value {
    fn to_f32(&self) -> f32 {
        self.as_f64().unwrap() as f32
    }

    fn to_u32(&self) -> u32 {
        self.as_u64().unwrap() as u32
    }

    fn as_opt(&self) -> Option<&Value> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    fn strip_nulls(&mut self) {
        match self {
            Value::Array(array) => array.iter_mut().for_each(|i| {
                i.strip_nulls();
            }),

            Value::Object(map) => {
                let mut nulls: Vec<String> = Vec::new();

                for (key, value) in map.iter_mut() {
                    match value {
                        Value::Null => nulls.push(key.clone()),
                        _ => value.strip_nulls(),
                    }
                }

                for key in nulls {
                    map.remove(&key);
                }
            }

            _ => {}
        }
    }
}
