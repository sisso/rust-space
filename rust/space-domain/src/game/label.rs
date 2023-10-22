use specs::prelude::*;

#[derive(Component)]
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
