use shred::{Read, ResourceId, SystemData, World, Write};
use specs::prelude::*;
use specs_derive::*;

use super::*;
use std::borrow::BorrowMut;
use crate::game::extractables::Extractable;

pub struct IndexPerSectorSystem;

#[derive(SystemData)]
pub struct IndexPerSectorData<'a> {
    entities: Entities<'a>,
    index: Write<'a, EntityPerSectorIndex>,
    locations: ReadStorage<'a, LocationSpace>,
    extractables: ReadStorage<'a, Extractable>,
}

impl<'a> System<'a> for IndexPerSectorSystem {
    type SystemData = IndexPerSectorData<'a>;

    fn run(&mut self, mut data: IndexPerSectorData) {
        use specs::Join;

        let index = data.index.borrow_mut();
        index.clear();

        for (entity, position) in (&data.entities, &data.locations).join() {
            index.add(position.sector_id, entity);

            if data.extractables.contains(entity) {
                index.add_extractable(position.sector_id, entity);
            }
        }
    }
}
