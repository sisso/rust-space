use crate::game_api::Id;
use godot::prelude::*;

#[derive(Clone, Debug, GodotClass)]
#[class(no_init)]
pub struct WareAmountInfo {
    pub id: Id,
    pub label: String,
    pub amount: i64,
}

#[godot_api]
impl WareAmountInfo {
    #[func]
    pub fn get_id(&self) -> Id {
        self.id
    }
    #[func]
    pub fn get_label(&self) -> String {
        self.label.clone()
    }
    #[func]
    pub fn get_amount(&self) -> i64 {
        self.amount
    }
}
