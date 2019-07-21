use crate::utils::{V2, Position};

#[derive(Clone,Debug)]
pub enum Action {
    Idle,
    Undock,
    Fly { to: Position },
    Jump,
    Mine,
}

pub struct Actions {

}

impl Actions {
    pub fn new() -> Self {
        Actions {

        }
    }
}
