use specs::prelude::*;
use crate::game::events::{Event, EventKind};
use crate::ffi::FfiOutpusBuilder;
use std::ops::DerefMut;
use std::borrow::{BorrowMut, Borrow};
use crate::game::locations::Location;

/// Convert Events into FFI outputs
pub struct FfiOutputSystem;

#[derive(SystemData)]
pub struct FfiOutputData<'a> {
    entities: Entities<'a>,
    events: ReadStorage<'a, Event>,
    output: Write<'a, Option<FfiOutpusBuilder>>,
    location: ReadStorage<'a, Location>
}

impl<'a> System<'a> for FfiOutputSystem {
    type SystemData = FfiOutputData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut output = FfiOutpusBuilder::new();

        for (id, event) in (&*data.entities, &data.events).join() {
            match event.kind {
                EventKind::Move => {
                    match data.location.borrow().get(event.id) {
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

        data.output.borrow_mut().replace(output);
    }
}
