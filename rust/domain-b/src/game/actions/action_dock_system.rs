use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use crate::game::dock::HasDocking;
use std::borrow::BorrowMut;

pub struct DockSystem;

#[derive(SystemData)]
pub struct DockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_dock: WriteStorage<'a, ActionDock>,
    location_space: WriteStorage<'a, LocationSpace>,
    location_docked: WriteStorage<'a, LocationDocked>,
    docking: WriteStorage<'a, HasDocking>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for DockSystem {
    type SystemData = DockData<'a>;

    fn run(&mut self, mut data: DockData) {
        log::trace!("running");

        let mut processed: Vec<(Entity, LocationDocked)> = vec![];

        for (entity, action, _) in (&*data.entities, &data.actions, &data.actions_dock).join() {
            let target_id = match action.get_action() {
                Action::Dock { target_id } => target_id.clone(),
                _ => continue,
            };

            log::debug!("{:?} docked at {:?}", entity, target_id);

            // update entity location
            processed.push((
                entity,
                LocationDocked {
                    parent_id: target_id,
                },
            ));

            // update docked list
            data.docking
                .get_mut(target_id)
                .expect("docked station has no docking component")
                .docked
                .push(entity);
        }

        let events = &mut data.events;

        for (entity, location) in processed {
            data.actions.borrow_mut().remove(entity).unwrap();
            data.actions_dock.borrow_mut().remove(entity).unwrap();
            data.location_space.borrow_mut().remove(entity).unwrap();
            data.location_docked
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
    use crate::game::dock::HasDocking;

    use crate::test::test_system;
    use crate::utils::Position;

    #[test]
    fn test_dock_system_should_dock() {
        let (world, (entity, station_id)) = test_system(DockSystem, |world| {
            let station_position = Position::ZERO;

            let sector_0 = world.create_entity().build();

            let station = world
                .create_entity()
                .with(LocationSpace {
                    pos: station_position,
                    sector_id: sector_0,
                })
                .with(HasDocking::default())
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Dock { target_id: station }))
                .with(ActionDock)
                .with(LocationSpace {
                    pos: station_position,
                    sector_id: sector_0,
                })
                .build();

            (entity, station)
        });

        // check
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionDock>().get(entity).is_none());
        let storage = world.read_storage::<LocationDocked>();
        match storage.get(entity) {
            Some(LocationDocked { parent_id }) => assert_eq!(*parent_id, station_id),
            _ => panic!(),
        }

        // check if docked object contain the new obj
        let station_has_dock = world
            .read_storage::<HasDocking>()
            .get(station_id)
            .unwrap()
            .clone();
        assert_eq!(1, station_has_dock.docked.len());
        assert_eq!(entity, station_has_dock.docked[0]);
    }
}
