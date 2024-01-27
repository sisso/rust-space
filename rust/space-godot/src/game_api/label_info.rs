use crate::game_api::{GameApi, Id};
use godot::prelude::*;

#[derive(Clone, Debug, GodotClass)]
pub struct LabelInfo {
    pub id: Id,
    pub label: String,
}

#[godot_api]
impl LabelInfo {
    #[func]
    pub fn get_id(&self) -> Id {
        self.id
    }
    #[func]
    pub fn get_label(&self) -> String {
        self.label.clone()
    }

    #[func]
    pub fn _to_string(&self) -> String {
        format!("LabelInfo(id: {}, label: {})", self.id, self.label)
    }
}

#[godot_api]
impl INode for LabelInfo {
    fn to_string(&self) -> GString {
        format!("LabelInfo(id: {}, label: {})", self.id, self.label).into()
    }
}
