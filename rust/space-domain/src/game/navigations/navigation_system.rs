use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::actions::*;

///
/// Execute actions for each NavigationMoveto without Action
///
pub struct NavigationSystem;

#[derive(SystemData)]
pub struct NavigationData<'a> {
    entities: Entities<'a>,
    navigation: WriteStorage<'a, Navigation>,
    navigation_move_to: WriteStorage<'a, NavigationMoveTo>,
    action: ReadStorage<'a, ActionActive>,
    action_request: WriteStorage<'a, ActionRequest>,
}

impl<'a> System<'a> for NavigationSystem {
    type SystemData = NavigationData<'a>;

    fn run(&mut self, mut data: NavigationData) {
        debug!("running");

        let mut completed = vec![];
        let mut requests = vec![];

        for (entity, nav, _) in (&*data.entities, &mut data.navigation_move_to, !&data.action).join() {
            match nav.next() {
                Some(action) => requests.push((entity, ActionRequest(action))),
                None => completed.push(entity),
            }
        }

        let requests_storage = data.action_request.borrow_mut();
        for (entity, action) in requests {
            debug!("{:?} request next action {:?}", entity, action);
            let _ = requests_storage.insert(entity, action).unwrap();
        }

        let navigation = data.navigation.borrow_mut();
        let navigation_move_to_storage = data.navigation_move_to.borrow_mut();
        for entity in completed {
            debug!("{:?} complete navigation", entity);

            let _ = navigation.remove(entity).unwrap();
            let _ = navigation_move_to_storage.remove(entity).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;

    #[test]
    fn test_navigation_move_to_system_should_complete_when_path_is_empty() {
        let (world, (entity, target)) =
            test_system(NavigationSystem, |world| {
                let target = world.create_entity()
                    .build();

                let entity = world.create_entity()
                    .with(Navigation::MoveTo)
                    .with(NavigationMoveTo { target: target, plan: NavigationPlan {
                        target_sector_id: SectorId(0),
                        target_position: Position::new(0.0, 0.0),
                        path: Default::default()
                    } })
                    .build();

                (entity, target)
            });

        let nav_storage = world.read_component::<Navigation>();
        assert!(nav_storage.get(entity).is_none());

        let nav_move_to_storage = world.read_component::<NavigationMoveTo>();
        assert!(nav_move_to_storage.get(entity).is_none());
    }
}
