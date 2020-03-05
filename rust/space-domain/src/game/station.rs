use specs::prelude::*;

// TODO: not in used
/// Static object that usually contains docks, storage, factories and shipyards
#[derive(Clone, Debug, Component)]
pub struct Station {

}

impl Station {
    pub fn new() -> Self {
        Station {}
    }
}

pub struct Stations;

impl Stations {
    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {

    }
}


