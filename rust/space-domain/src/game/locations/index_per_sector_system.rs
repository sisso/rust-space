use super::*;
use crate::game::extractables::Extractable;
use shred::{Read, ResourceId, SystemData, World, Write};
use specs::prelude::*;
use specs_derive::*;
use std::borrow::BorrowMut;

/// Index entities to provide fast look up like:
/// - what ships are in sector 0?
/// - what is nearest asteroid from sector 2?
// TODO: optimize to only update elements that have changed. Add SectorChange flag
pub struct IndexPerSectorSystem;

#[derive(SystemData)]
pub struct IndexPerSectorData<'a> {
    entities: Entities<'a>,
    index: Write<'a, EntityPerSectorIndex>,
    locations: ReadStorage<'a, Location>,
    extractables: ReadStorage<'a, Extractable>,
}

impl<'a> System<'a> for IndexPerSectorSystem {
    type SystemData = IndexPerSectorData<'a>;

    fn run(&mut self, mut data: IndexPerSectorData) {
        use specs::Join;

        trace!("running");

        let index = data.index.borrow_mut();
        index.clear();

        for (entity, location) in (&data.entities, &data.locations).join() {
            match location {
                Location::Space { sector_id, .. } => {
                    let sector_id = *sector_id;

                    trace!("indexing {:?} at {:?}", entity, sector_id);
                    index.add(sector_id, entity);

                    if data.extractables.contains(entity) {
                        trace!("indexing extractable {:?} at {:?}", entity, sector_id);
                        index.add_extractable(sector_id, entity);
                    }
                }
                _ => {}
            }
        }
    }
}
