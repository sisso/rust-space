use specs::prelude::*;

use crate::game::wares::WareId;

#[derive(Clone, Debug, Component)]
pub struct Extractable {
    pub ware_id: WareId,
}

impl Extractable {
    pub fn list(world: &World) -> Vec<(Entity, Extractable)> {
        let entities = world.entities();
        let extractables = world.read_component::<Extractable>();

        let mut result = vec![];
        for (id, ext) in (&entities, &extractables).join() {
            result.push((id, ext.clone()));
        }

        result
    }
}
