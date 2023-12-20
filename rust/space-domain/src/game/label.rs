use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Label {
    pub label: String,
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Label {
            label: value.to_string(),
        }
    }
}
