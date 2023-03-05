use crate::ffi::FfiOutpusBuilder;
use crate::game::events::{EventKind, Events};
use crate::game::extractables::Extractable;
use crate::game::locations::Location;
use crate::game::sectors::{Jump, Sector};
use crate::game::station::Station;
use crate::game::wares::Ware;
use crate::space_outputs_generated::space_data::{EntityKind, JumpNew, SectorNew};
use crate::utils::IdAsU32Support;

use specs::prelude::*;

/// Convert Events into FFI outputs
#[derive(Default)]
pub struct FfiOutputSystem;

#[derive(SystemData)]
pub struct FfiOutputData<'a> {
    entities: Entities<'a>,
    events: Write<'a, Events>,
    output: Write<'a, FfiOutpusBuilder>,
    location: ReadStorage<'a, Location>,
    station: ReadStorage<'a, Station>,
    sectors: ReadStorage<'a, Sector>,
    jumps: ReadStorage<'a, Jump>,
    wares: ReadStorage<'a, Ware>,
    extractable: ReadStorage<'a, Extractable>,
}

impl FfiOutputSystem {
    fn resolve_entity_kind(
        entity: Entity,
        station: &ReadStorage<Station>,
        extractable: &ReadStorage<Extractable>,
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

        for event in data.events.take() {
            let entity = event.id;

            match &event.kind {
                EventKind::Add => {
                    if data.wares.get(entity).is_some() {
                        // ignore for now
                    } else if let Some(sector) = data.sectors.get(entity) {
                        output
                            .sectors_new
                            .push(SectorNew::new(entity.as_u32(), &sector.coords.into()));
                    } else if let Some(jump) = data.jumps.get(entity) {
                        let jump_location = data.location.get(entity).unwrap().as_space().unwrap();

                        output.jumps_new.push(JumpNew::new(
                            entity.as_u32(),
                            jump_location.sector_id.as_u32(),
                            &jump_location.pos.into(),
                            jump.target_sector_id.as_u32(),
                            &jump.target_pos.into(),
                        ));
                    } else {
                        let entity_kind = FfiOutputSystem::resolve_entity_kind(
                            entity,
                            &data.station,
                            &data.extractable,
                        );

                        match data.location.get(entity) {
                            Some(Location::Space { pos, sector_id }) => {
                                output.push_entity_new_in_space(
                                    entity.id(),
                                    entity_kind,
                                    pos.into(),
                                    sector_id.as_u32(),
                                );
                            }
                            Some(Location::Dock { docked_id }) => {
                                output.push_entity_new_docked(
                                    entity.id(),
                                    entity_kind,
                                    docked_id.as_u32(),
                                );
                            }
                            other => {
                                // ignore entities not visible in sector
                                log::warn!(
                                    "{:?} added entity with invalid position {:?}",
                                    entity,
                                    other,
                                );
                            }
                        }
                    }
                }

                EventKind::Dock => {
                    match data.location.get(entity) {
                        Some(Location::Dock { docked_id }) => {
                            output.push_entity_dock(entity.id(), docked_id.as_u32());
                        }
                        other => {
                            // ignore entities not visible in sector
                            log::warn!("{:?} undock but has no space {:?}", entity, other);
                        }
                    }
                }

                EventKind::Undock => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            output.push_entity_undock(entity.id(), pos.into(), sector_id.as_u32());
                        }
                        other => {
                            // ignore entities not visible in sector
                            log::warn!("{:?} undock but has no space {:?}", entity, other);
                        }
                    }
                }

                EventKind::Jump => {
                    match data.location.get(entity) {
                        Some(Location::Space { pos, sector_id }) => {
                            output.push_entity_jump(entity.id(), pos.into(), sector_id.as_u32());
                        }
                        other => {
                            // ignore entities not visible in sector
                            log::warn!("{:?} jump but has no space {:?}", entity, other);
                        }
                    }
                }

                EventKind::Move => match data.location.get(entity) {
                    Some(Location::Space { pos, .. }) => {
                        output.push_entity_move(entity.id(), pos.into());
                    }
                    other => {
                        log::warn!("{:?} moved but has no space position {:?}", entity, other);
                    }
                },

                _other => {
                    log::debug!("unknown event {:?}", event);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::events::Event;

    use crate::test::test_system;
    use crate::utils::V2;
    use std::borrow::BorrowMut;

    #[test]
    fn test_ffi_output_system_added_docked_create_new_entity_and_dock_output() {
        let (world, (id, station_id)) = test_system(FfiOutputSystem, |world| {
            let arbitrary_station = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Dock {
                    docked_id: arbitrary_station,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Add));

            (arbitrary_entity.id(), arbitrary_station.id())
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();

        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());

        let entry = output.entities_dock.iter().next().unwrap();
        assert_eq!(entry.target_id(), station_id);
    }

    #[test]
    fn test_ffi_output_system_added_entity_in_space_should_create_new_entity_and_teleport_output() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Add));

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();

        let entry = output.entities_new.iter().next().unwrap();
        assert_eq!(id, entry.id());

        let entry = output.entities_teleport.iter().next().unwrap();
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), sector_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_undock_event_should_generate_undock() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Undock));

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_undock.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
        assert_eq!(entry.sector_id(), sector_0.as_u32());
    }

    #[test]
    fn test_ffi_output_system_jump_event_should_generate_jump_output() {
        let (world, (id, sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Jump));

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
        let (world, (id, _sector_0)) = test_system(FfiOutputSystem, |world| {
            let sector_0 = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Space {
                    pos: V2::new(1.0, 2.0),
                    sector_id: sector_0,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Move));

            (arbitrary_entity.id(), sector_0)
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();
        let entry = output.entities_moved.iter().next().unwrap();
        assert_eq!(id, entry.id());
        assert_eq!(entry.pos().x(), 1.0);
        assert_eq!(entry.pos().y(), 2.0);
    }

    #[test]
    fn test_ffi_output_system_docked_should_create_dock_output() {
        let (world, (_id, station_id)) = test_system(FfiOutputSystem, |world| {
            let arbitrary_station = world.create_entity().build();

            let arbitrary_entity = world
                .create_entity()
                .with(Location::Dock {
                    docked_id: arbitrary_station,
                })
                .build();

            world
                .write_resource::<Events>()
                .borrow_mut()
                .push(Event::new(arbitrary_entity, EventKind::Dock));

            (arbitrary_entity.id(), arbitrary_station.id())
        });

        let output = &*world.read_resource::<FfiOutpusBuilder>();

        let entry = output.entities_dock.iter().next().unwrap();
        assert_eq!(entry.target_id(), station_id);
    }
}
