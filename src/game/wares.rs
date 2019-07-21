use crate::utils::*;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct WareId(pub u32);

#[derive(Clone,Debug)]
pub struct Extractable {
    pub ware_id: WareId,
    pub time: Seconds,
}

#[derive(Debug, Clone)]
pub struct Cargo {
    pub max: u32
}

impl Cargo {
    pub fn new(max: u32) -> Self {
        Cargo {
            max
        }
    }
}
