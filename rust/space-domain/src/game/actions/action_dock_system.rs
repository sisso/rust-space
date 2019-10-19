use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::actions::*;
use crate::game::objects::HasDock;

pub struct DockSystem;

#[derive(SystemData)]
pub struct DockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, Action>,
    actions_dock: WriteStorage<'a, ActionDock>,
    locations_dock: WriteStorage<'a, LocationDock>,
    locations_space: WriteStorage<'a, LocationSpace>,
}

impl<'a> System<'a> for DockSystem {
    type SystemData = DockData<'a>;

    fn run(&mut self, mut data: DockData) {
        let mut processed: Vec<(Entity, LocationDock)> = vec![];

        for (entity, action, dock) in (&*data.entities, &data.actions, &data.actions_dock).join() {
            let target_id = match action.request {
                ActionRequest::Dock { target_id } => target_id,
                _ => continue,
            };

            processed.push((entity, LocationDock { docked_id: target_id } ));
        }

        for (entity, location) in processed {
            let _ = data.actions.borrow_mut().remove(entity);
            let _ = data.actions_dock.borrow_mut().remove(entity);
            let _ = data.locations_space.borrow_mut().remove(entity);
            let _ = data.locations_dock.borrow_mut().insert(entity, location);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::test::{test_system, assert_v2};

    #[test]
    fn test_dock_system_should_dock() {
        let (world, (entity, station)) = test_system(DockSystem, |world| {
            let station_position = Position::new(0.0, 0.0);

            let station = world.create_entity()
                .with(LocationSpace { pos: station_position })
                .build();

            let entity = world.create_entity()
                .with(Action { request: ActionRequest::Dock { target_id: station } })
                .with(ActionDock)
                .with(LocationSpace { pos: station_position })
                .build();

            (entity, station)
        });

        assert!(world.read_storage::<Action>().get(entity).is_none());
        assert!(world.read_storage::<ActionDock>().get(entity).is_none());
        let storage = world.read_storage::<LocationDock>();
        match storage.get(entity) {
            Some(LocationDock { docked_id }) => {
                assert_eq!(*docked_id, station)
            },
            _ => panic!()
        }
    }
}
