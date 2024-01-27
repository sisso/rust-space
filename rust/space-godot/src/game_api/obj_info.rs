use super::shipyard_info::ShipyardInfo;
use super::ware_amount_info::WareAmountInfo;
use crate::game_api::label_info::LabelInfo;
use crate::game_api::Id;
use godot::prelude::*;

#[derive(Clone, Debug, GodotClass)]
pub struct ObjExtendedInfo {
    pub id: Id,
    pub label: String,
    pub pos: Vector2,
    pub kind: String,
    pub is_star: bool,
    pub is_fleet: bool,
    pub is_planet: bool,
    pub is_asteroid: bool,
    pub is_jump: bool,
    pub is_station: bool,
    pub is_orbiting: bool,
    pub shipyard: Option<Gd<ShipyardInfo>>,
    pub orbit_parent_id: Id,
    pub command: String,
    pub action: String,
    pub cargo: Vec<WareAmountInfo>,
    pub requesting_wares: Array<Gd<LabelInfo>>,
    pub providing_wares: Array<Gd<LabelInfo>>,
}

#[godot_api]
impl ObjExtendedInfo {
    #[func]
    pub fn get_id(&self) -> Id {
        self.id
    }
    #[func]
    pub fn get_label(&self) -> String {
        self.label.clone()
    }
    #[func]
    pub fn get_pos(&self) -> Vector2 {
        self.pos
    }
    #[func]
    pub fn get_kind(&self) -> String {
        self.kind.clone()
    }
    #[func]
    pub fn is_star(&self) -> bool {
        self.is_star
    }
    #[func]
    pub fn is_fleet(&self) -> bool {
        self.is_fleet
    }
    #[func]
    pub fn is_planet(&self) -> bool {
        self.is_planet
    }
    #[func]
    pub fn is_asteroid(&self) -> bool {
        self.is_asteroid
    }
    #[func]
    pub fn is_jump(&self) -> bool {
        self.is_jump
    }
    #[func]
    pub fn is_station(&self) -> bool {
        self.is_station
    }
    #[func]
    pub fn is_orbiting(&self) -> bool {
        self.is_orbiting
    }
    #[func]
    pub fn get_shipyard(&self) -> Option<Gd<ShipyardInfo>> {
        self.shipyard.clone()
    }
    #[func]
    pub fn get_orbit_parent_id(&self) -> Id {
        self.orbit_parent_id
    }
    #[func]
    pub fn get_command(&self) -> String {
        self.command.clone()
    }
    #[func]
    pub fn get_action(&self) -> String {
        self.action.clone()
    }
    #[func]
    pub fn get_cargo_size(&self) -> i32 {
        self.cargo.len() as i32
    }
    #[func]
    pub fn get_cargo(&self, index: i32) -> Gd<WareAmountInfo> {
        let wi = self.cargo[index as usize].clone();
        Gd::from_object(wi)
    }
    #[func]
    pub fn get_requesting_wares(&self) -> Array<Gd<LabelInfo>> {
        self.requesting_wares.clone()
    }
    #[func]
    pub fn get_providing_wares(&self) -> Array<Gd<LabelInfo>> {
        self.providing_wares.clone()
    }
}
