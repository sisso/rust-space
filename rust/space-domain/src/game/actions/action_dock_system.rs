use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use std::borrow::BorrowMut;

pub struct DockSystem;

#[derive(SystemData)]
pub struct DockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_dock: WriteStorage<'a, ActionDock>,
    locations: WriteStorage<'a, Location>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for DockSystem {
    type SystemData = DockData<'a>;

    fn run(&mut self, mut data: DockData) {
        log::trace!("running");

        let mut processed: Vec<(Entity, Location)> = vec![];

        for (entity, action, _dock) in (&*data.entities, &data.actions, &data.actions_dock).join() {
            let target_id = match action.get_action() {
                Action::Dock { target_id } => target_id.clone(),
                _ => continue,
            };

            log::debug!("{:?} docked at {:?}", entity, target_id);
            processed.push((
                entity,
                Location::Dock {
                    docked_id: target_id,
                },
            ));
        }

        let events = &mut data.events;
        for (entity, location) in processed {
            data.actions.borrow_mut().remove(entity).unwrap();
            data.actions_dock.borrow_mut().remove(entity).unwrap();
            data.locations
                .borrow_mut()
                .insert(entity, location)
                .unwrap();
            events.push(Event::new(entity, EventKind::Dock));
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;

    use crate::test::test_system;
    use crate::utils::Position;

    #[test]
    fn test_dock_system_should_dock() {
        let (world, (entity, station)) = test_system(DockSystem, |world| {
            let station_position = Position::ZERO;

            let sector_0 = world.create_entity().build();

            let station = world
                .create_entity()
                .with(Location::Space {
                    pos: station_position,
                    sector_id: sector_0,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Dock { target_id: station }))
                .with(ActionDock)
                .with(Location::Space {
                    pos: station_position,
                    sector_id: sector_0,
                })
                .build();

            (entity, station)
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionDock>().get(entity).is_none());
        let storage = world.read_storage::<Location>();
        match storage.get(entity) {
            Some(Location::Dock { docked_id }) => assert_eq!(*docked_id, station),
            _ => panic!(),
        }
    }
}
