use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use crate::game::dock::Docking;
use std::borrow::{Borrow, BorrowMut};

pub struct UndockSystem;

#[derive(SystemData)]
pub struct UndockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_undock: WriteStorage<'a, ActionUndock>,
    locations: WriteStorage<'a, Location>,
    docking: WriteStorage<'a, Docking>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for UndockSystem {
    type SystemData = UndockData<'a>;

    fn run(&mut self, mut data: UndockData) {
        log::trace!("running");

        let mut processed: Vec<(Entity, Option<(Location, Entity)>)> = vec![];

        for (entity, _, location) in (&*data.entities, &data.actions_undock, &data.locations).join()
        {
            if let Some(docked_id) = location.as_docked() {
                let target_location = data.locations.get(docked_id);
                match target_location {
                    Some(location @ Location::Space { .. }) => {
                        log::debug!("{:?} un-docking from {:?}", entity, docked_id);
                        processed.push((entity, Some((location.clone(), docked_id))));
                    }
                    _ => {
                        log::warn!(
                            "{:?} can not un-dock from {:?}, target has no location",
                            entity,
                            target_location
                        );
                        processed.push((entity, None))
                    }
                }
            } else {
                log::debug!(
                    "{:?} can not un-dock, ship is already in space {:?}, ignoring",
                    entity,
                    location,
                );
                processed.push((entity, None));
            }
        }

        let events = &mut data.events;

        for (entity, location_and_docked_id) in processed {
            if let Some((location, docked_id)) = location_and_docked_id {
                // update entity location
                data.locations.insert(entity, location).unwrap();

                // remove entity from docked
                data.docking
                    .get_mut(docked_id)
                    .expect("docked entity has no docking")
                    .docked
                    .retain(|id| *id != entity);
            }
            data.actions.remove(entity).unwrap();
            data.actions_undock.remove(entity).unwrap();
            events.push(Event::new(entity, EventKind::Undock));
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;

    use crate::test::{assert_v2, test_system};
    use crate::utils::Position;

    #[test]
    fn test_undock_system_should_undock_if_docked() {
        let (world, (fleet_id, station_id)) = test_system(UndockSystem, |world| {
            let sector_id = world.create_entity().build();

            let station_id = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id: sector_id,
                })
                .with(Docking::default())
                .build();

            let fleet_id = world
                .create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(Location::Dock {
                    docked_id: station_id,
                })
                .build();

            // update station docking
            world
                .write_storage::<Docking>()
                .get_mut(station_id)
                .unwrap()
                .docked
                .push(fleet_id);

            (fleet_id, station_id)
        });

        // check fleet
        assert!(world.read_storage::<ActionActive>().get(fleet_id).is_none());
        assert!(world.read_storage::<ActionUndock>().get(fleet_id).is_none());
        let storage = world.read_storage::<Location>();
        let position = storage.get(fleet_id);
        match position {
            Some(Location::Space { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }

        // check docking
        assert!(world
            .read_storage::<Docking>()
            .get(station_id)
            .unwrap()
            .docked
            .is_empty());
    }

    #[test]
    fn test_undock_system_should_ignore_undock_if_not_docked() {
        let (world, entity) = test_system(UndockSystem, |world| {
            let sector_id = world.create_entity().build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id,
                })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        let storage = world.read_storage::<Location>();
        let position = storage.get(entity);
        match position {
            Some(Location::Space { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }
    }
}
