use specs::prelude::*;

use super::super::locations::*;
use super::*;
use crate::game::actions::*;
use std::borrow::{Borrow, BorrowMut};

pub struct UndockSystem;

#[derive(SystemData)]
pub struct UndockData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_undock: WriteStorage<'a, ActionUndock>,
    locations: WriteStorage<'a, Location>,
}

impl<'a> System<'a> for UndockSystem {
    type SystemData = UndockData<'a>;

    fn run(&mut self, mut data: UndockData) {
        trace!("running");

        let mut processed: Vec<(Entity, Option<Location>)> = vec![];

        let locations_storage = data.locations.borrow();

        for (entity, _, location) in (
            &*data.entities,
            &data.actions_undock,
            &data.locations,
        )
            .join()
        {
            if let Some(docked_id) = location.as_docked() {
                let target_location = locations_storage.get(docked_id);
                match target_location {
                    Some(location @ Location::Space { .. }) => {
                        debug!("{:?} undocking", entity);
                        processed.push((entity, Some(location.clone())));
                    },
                    _ => {
                        debug!("{:?} can not undock at {:?}", entity, target_location);
                        processed.push((entity, None))
                    },
                }
            } else {
                debug!("{:?} can not undock, it is not docked", entity);
                processed.push((entity, None));
            }
        }

        for (entity, location) in processed {
            if let Some(location) = location {
                data.locations.borrow_mut().insert(entity, location).unwrap();
            }
            let _ = data.actions.borrow_mut().remove(entity);
            let _ = data.actions_undock.borrow_mut().remove(entity);
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::test::{assert_v2, test_system};
    use crate::game::sectors::SectorId;

    pub const SECTOR_0: SectorId = SectorId(0);

    #[test]
    fn test_undock_system_should_undock_if_docked() {
        let (world, entity) = test_system(UndockSystem, |world| {
            let station = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id: SECTOR_0,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(Location::Dock { docked_id: station })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        assert!(world.read_storage::<Location>().get(entity).is_none());
        let storage = world.read_storage::<Location>();
        let position = storage.get(entity);
        match position {
            Some(Location::Space { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_undock_system_should_ignore_undock_if_not_docked() {
        let (world, entity) = test_system(UndockSystem, |world| {
            let entity = world
                .create_entity()
                .with(ActionActive(Action::Undock))
                .with(ActionUndock)
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id: SECTOR_0,
                })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionUndock>().get(entity).is_none());
        assert!(world.read_storage::<Location>().get(entity).is_none());
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
