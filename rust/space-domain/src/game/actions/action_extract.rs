use specs::prelude::*;
use super::objects::ObjId;

#[derive(Clone,Debug,Component)]
pub struct ActionExtract {
    pub target_id: ObjId,
    pub complete_time: TotalTime,
}

