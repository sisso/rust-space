use specs::prelude::*;
use crate::game::events::{Event, EventKind};
use crate::ffi::FfiOutpusBuilder;
use std::ops::DerefMut;
use std::borrow::{BorrowMut, Borrow};
use crate::game::locations::Location;
use crate::space_outputs_generated::space_data::{V2, EntityKind};
use crate::game::station::Station;
use crate::game::extractables::Extractable;

/// Convert Events into FFI outputs
pub struct FfiOutputSystem;

#[derive(SystemData)]
pub struct FfiOutputData<'a> {
    entities: Entities<'a>,
    events: ReadStorage<'a, Event>,
    output: Write<'a, FfiOutpusBuilder>,
    location: ReadStorage<'a, Location>,
    station: ReadStorage<'a, Station>,
    extractable: ReadStorage<'a, Extractable>,
}

impl<'a> System<'a> for FfiOutputSystem {
    type SystemData = FfiOutputData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let output = &mut data.output;
        output.clear();

        for (_event_entity, event) in (&*data.entities, &data.events).join() {
            let entity = event.id;

            // TODO: optimize to fetch only when it is needed
            let entity_kind =
                if data.station.get(entity).is_some() {
                    EntityKind::Station
                } else if data.extractable.get(entity).is_some() {
                    EntityKind::Asteroid
                } else {
                    EntityKind::Fleet
                };

            match &event.kind {
                EventKind::Add => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            output.push_entity_new(entity.id(), pos.into(), sector_id.as_u32(), entity_kind);
                        },
                        other => {
                            // ignore entities not visible in sector
                        },
                    }
                },

                EventKind::Undock => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            output.push_entity_new(entity.id(), pos.into(), sector_id.as_u32(), entity_kind);
                        },
                        other => {
                            // ignore entities not visible in sector
                            warn!("{:?} undock but has no space {:?}", entity, other);
                        },
                    }
                },

                EventKind::Jump => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            output.push_entity_jump(entity.id(), pos.into(), sector_id.as_u32());
                        },
                        other => {
                            // ignore entities not visible in sector
                            warn!("{:?} jump but has no space {:?}", entity, other);
                        },
                    }
                },

                EventKind::Move => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, .. }) => {
                            output.push_entity_move(entity.id(), pos.into());
                        },
                        other => {
                          warn!("{:?} moved but has no space position {:?}", entity, other);
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
    use crate::game::sectors::SectorId;
    use crate::utils::V2;

    const SECTOR_0: SectorId = SectorId(0);

    #[test]
    fn test_ffi_output_system_added_docked_not_create_new_entity() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_station = world.create_entity().build();

            let arbitrary_entity = world.create_entity()
                .with(Location::Dock { docked_id: arbitrary_station, })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Add))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        assert_eq!(output.entities_new.iter().next(), None);
    }
    #[test]
    fn test_ffi_output_system_added_entity_in_space() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: SECTOR_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Add))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), SECTOR_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_undock_event_should_generate_added_output() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: SECTOR_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Undock))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), SECTOR_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_jump_event_should_generate_jump_output() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: SECTOR_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Jump))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_jumped.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), SECTOR_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_moved_should_generate_move_output() {
        let (world, id) = test_system(FfiOutputSystem, |world| {
            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: SECTOR_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Move))
                .build();

            arbitrary_entity.id()
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_moved.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
    }
}
