use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use crate::game::dock::HasDocking;

pub struct UndockSystem;

#[derive(SystemData)]
pub struct UndockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_undock: WriteStorage<'a, ActionUndock>,
    locations_space: WriteStorage<'a, LocationSpace>,
    locations_docked: WriteStorage<'a, LocationDocked>,
    has_docking: WriteStorage<'a, HasDocking>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for UndockSystem {
    type SystemData = UndockData<'a>;

    fn run(&mut self, mut data: UndockData) {
        log::trace!("running");

        let mut processed: Vec<(Entity, Option<(LocationSpace, Entity)>)> = vec![];

        for (id, _, docked_at) in (
            &*data.entities,
            &data.actions_undock,
            data.locations_docked.maybe(),
        )
            .join()
        {
            if let Some(docked_at) = docked_at {
                match data.locations_space.get(docked_at.parent_id) {
                    Some(location) => {
                        log::debug!("{:?} un-docking from {:?}", id, docked_at.parent_id);
                        processed.push((id, Some((location.clone(), docked_at.parent_id))));
                    }
                    None => {
                        log::warn!("{:?} can not un-dock, parent is not in space", id,);
                        processed.push((id, None))
                    }
                }
            } else {
                log::warn!("{:?} can no undock, it is not docked, ignoring", id);
                processed.push((id, None))
            }
        }

        let events = &mut data.events;

        for (entity, location_and_docked_id) in processed {
            if let Some((new_location, docked_id)) = location_and_docked_id {
                // update entity location
                data.locations_space.insert(entity, new_location).unwrap();
                data.locations_docked.remove(entity).unwrap();

                // remove entity from docked
                data.has_docking
                    .get_mut(docked_id)
                    .expect("docked entity has no docking")
                    .docked
                    .retain(|id| *id != entity);
            }
            // remove actions
            data.actions.remove(entity).unwrap();
            data.actions_undock.remove(entity).unwrap();
            // generate events
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
                .with(LocationSpace {
                    pos: Position::new(0.0, 0.0),
                    sector_id: sector_id,
                })
                .with(HasDocking::default())
                .build();

            let fleet_id = world
                .create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(LocationDocked {
                    parent_id: station_id,
                })
                .build();

            // update station docking
            world
                .write_storage::<HasDocking>()
                .get_mut(station_id)
                .unwrap()
                .docked
                .push(fleet_id);

            (fleet_id, station_id)
        });

        // check fleet
        assert!(world.read_storage::<ActionActive>().get(fleet_id).is_none());
        assert!(world.read_storage::<ActionUndock>().get(fleet_id).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let position = storage.get(fleet_id);
        match position {
            Some(LocationSpace { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }

        // check docking
        assert!(world
            .read_storage::<HasDocking>()
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
                .with(LocationSpace {
                    pos: Position::new(0.0, 0.0),
                    sector_id,
                })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let position = storage.get(entity);
        match position {
            Some(LocationSpace { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }
    }
}
