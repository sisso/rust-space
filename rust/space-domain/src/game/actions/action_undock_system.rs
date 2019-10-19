use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::actions::*;

pub struct UndockSystem;

#[derive(SystemData)]
pub struct UndockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_undock: WriteStorage<'a, ActionUndock>,
    locations_dock: WriteStorage<'a, LocationDock>,
    locations_space: WriteStorage<'a, LocationSpace>,
}

impl<'a> System<'a> for UndockSystem {
    type SystemData = UndockData<'a>;

    fn run(&mut self, mut data: UndockData) {
        let mut processed: Vec<(Entity, Option<LocationSpace>)> = vec![];

        let location_space_storage = data.locations_space
            .borrow();

        for (entity, _, location_dock) in (&*data.entities, &data.actions_undock, data.locations_dock.maybe()).join() {
            let location_space = match location_dock {
                Some(location_dock) => {
                    location_space_storage
                        .get(location_dock.docked_id)
                        .map(|value| value.clone())
                },
                None => None
            };

            processed.push((entity, location_space));
        }

        for (entity, location) in processed {
            if let Some(location) = location {
                let _ = data.locations_space.borrow_mut().insert(entity, location);
            }
            let _ = data.locations_dock.borrow_mut().remove(entity);
            let _ = data.actions.borrow_mut().remove(entity);
            let _ = data.actions_undock.borrow_mut().remove(entity);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::test::{test_system, assert_v2};

    #[test]
    fn test_undock_system_should_undock_if_docked() {
        let (world, entity) = test_system(UndockSystem, |world| {
            let station = world.create_entity()
                .with(LocationSpace { pos: Position::new(0.0, 0.0) })
                .build();

            let entity = world.create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(LocationDock { docked_id: station })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        assert!(world.read_storage::<LocationDock>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let position = storage.get(entity);
        match position {
            Some(LocationSpace { pos }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            },
            _ => panic!()
        }
    }

    #[test]
    fn test_undock_system_should_ignore_undock_if_not_docked() {
        let (world, entity) = test_system(UndockSystem, |world| {
            let entity = world.create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(LocationSpace { pos: Position::new(0.0, 0.0) })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        assert!(world.read_storage::<LocationDock>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let position = storage.get(entity);
        match position {
            Some(LocationSpace { pos }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            },
            _ => panic!()
        }
    }
}
