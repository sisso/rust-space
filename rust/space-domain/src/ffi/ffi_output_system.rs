use specs::prelude::*;
use crate::game::events::{Event, EventKind};
use crate::ffi::FfiOutpusBuilder;
use std::ops::DerefMut;
use std::borrow::{BorrowMut, Borrow};
use crate::game::locations::Location;
use crate::space_outputs_generated::space_data::{V2, EntityKind};

/// Convert Events into FFI outputs
pub struct FfiOutputSystem;

#[derive(SystemData)]
pub struct FfiOutputData<'a> {
    entities: Entities<'a>,
    events: ReadStorage<'a, Event>,
    output: Write<'a, FfiOutpusBuilder>,
    location: ReadStorage<'a, Location>
}

impl<'a> System<'a> for FfiOutputSystem {
    type SystemData = FfiOutputData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let output = &mut data.output;
        output.clear();

        for (event_entity, event) in (&*data.entities, &data.events).join() {
            let id = event.id.id();

            match &event.kind {
                EventKind::Add => {
                    output.push_entity_new(id, V2::new(0.0, 0.0), 0, EntityKind::Fleet);
                },

                EventKind::Move => {
                    match data.location.borrow().get(event_entity) {
                        Some(Location::Space { pos, .. }) =>

                            output.push_entity_move(id, pos.into()),
                        other => {
                          warn!("{:?} moved but has no space position {:?}", event.id, other);
                        },
                    }
                },

                other => {
                    debug!("unknown event {:?}", event);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{assert_v2, test_system};

    #[test]
    fn test_ffi_output_system() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_entity = world.create_entity().build();
            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Add))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());

        unimplemented!("added is using dummy values");
    }
}
