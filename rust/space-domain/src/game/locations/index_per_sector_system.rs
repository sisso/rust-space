use shred::{Read, ResourceId, SystemData, World, Write};
use specs::prelude::*;
use specs_derive::*;
use super::*;
use std::borrow::BorrowMut;
use crate::game::extractables::Extractable;

/// Index entities to provide fast look up like:
/// - what ships are in sector 0?
/// - what is nearest asteroid from sector 2?
// TODO: optimize to only update elements that have changed. Add SectorChange flag
pub struct IndexPerSectorSystem;

#[derive(SystemData)]
pub struct IndexPerSectorData<'a> {
    entities: Entities<'a>,
    index: Write<'a, EntityPerSectorIndex>,
    locations_sector: ReadStorage<'a, LocationSector>,
    extractables: ReadStorage<'a, Extractable>,
}

impl<'a> System<'a> for IndexPerSectorSystem {
    type SystemData = IndexPerSectorData<'a>;

    fn run(&mut self, mut data: IndexPerSectorData) {
        use specs::Join;

        debug!("running");

        let index = data.index.borrow_mut();
        index.clear();

        for (entity, location_sector) in (&data.entities, &data.locations_sector).join() {
            debug!("indexing {:?} at {:?}", entity, location_sector.sector_id);
            index.add(location_sector.sector_id, entity);

            if data.extractables.contains(entity) {
                debug!("indexing extractable {:?} at {:?}", entity, location_sector.sector_id);
                index.add_extractable(location_sector.sector_id, entity);
            }
        }
    }
}
