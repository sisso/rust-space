use specs::prelude::*;
use crate::game::events::{Event, EventKind};
use crate::ffi::FfiOutpusBuilder;
use std::ops::DerefMut;
use std::borrow::{BorrowMut, Borrow};
use crate::game::locations::Location;
use crate::space_outputs_generated::space_data::{V2, EntityKind, SectorNew, JumpNew};
use crate::game::station::Station;
use crate::game::extractables::Extractable;
use crate::utils::IdAsU32Support;
use crate::game::sectors::{Sector, Jump};

/// Convert Events into FFI outputs
pub struct FfiOutputSystem;

#[derive(SystemData)]
pub struct FfiOutputData<'a> {
    entities: Entities<'a>,
    events: ReadStorage<'a, Event>,
    output: Write<'a, FfiOutpusBuilder>,
    location: ReadStorage<'a, Location>,
    station: ReadStorage<'a, Station>,
    sectors: ReadStorage<'a, Sector>,
    jumps: ReadStorage<'a, Jump>,
    extractable: ReadStorage<'a, Extractable>,
}

impl FfiOutputSystem {
    fn resolve_entity_kind(
        entity: Entity,
        station: &ReadStorage<Station>,
        extractable: &ReadStorage< Extractable>,
    ) -> EntityKind {
        if station.get(entity).is_some() {
            EntityKind::Station
        } else if extractable.get(entity).is_some() {
            EntityKind::Asteroid
        } else {
            EntityKind::Fleet
        }
    }
}

impl<'a> System<'a> for FfiOutputSystem {
    type SystemData = FfiOutputData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let output = &mut data.output;
        // TODO: the fetcher should be one that remove events
        output.clear();

        for (_event_entity, event) in (&*data.entities, &data.events).join() {
            let entity = event.id;

            match &event.kind {
                EventKind::Add => {
                    if data.sectors.get(entity).is_some() {
                        output.sectors_new.push(SectorNew::new(entity.as_u32()));
                    } else if let Some(jump) = data.jumps.get(entity) {
                        let jump_location = data.location.get(entity).unwrap().as_space().unwrap();
                        let target_jump = data.location.get(jump.target_id).unwrap().as_space().unwrap();

                        output.jumps_new.push(JumpNew::new(
                            entity.as_u32(),
                            jump_location.sector_id.as_u32(),
                            &jump_location.pos.into(),
                            target_jump.sector_id.as_u32(),
                            &target_jump.pos.into(),
                        ));
                    } else {
                        match data.location.get(entity) {
                            Some(Location::Space { pos, sector_id }) => {
                                let entity_kind = FfiOutputSystem::resolve_entity_kind(entity, &data.station, &data.extractable);
                                output.push_entity_new(entity.id(), pos.into(), sector_id.as_u32(), entity_kind);
                            },
                            Some(Location::Dock { .. }) => {
                                // ignore docked objects
                            },
                            other => {
                                // ignore entities not visible in sector
                                panic!("{:?} has no valid location {:?}", entity, other);
                            },
                        }
                    }
                },

                EventKind::Undock => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            let entity_kind = FfiOutputSystem::resolve_entity_kind(entity, &data.station, &data.extractable);
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
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Add))
                .build();

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), sector_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_undock_event_should_generate_added_output() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Undock))
                .build();

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), sector_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_jump_event_should_generate_jump_output() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Jump))
                .build();

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_jumped.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), sector_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_moved_should_generate_move_output() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world.create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world.create_entity()
                .with(Event::new(arbitrary_entity, EventKind::Move))
                .build();

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_moved.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
    }
}
