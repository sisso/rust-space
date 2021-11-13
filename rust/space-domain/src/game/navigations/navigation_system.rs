use log::{debug, info, log, trace, warn};
use specs::prelude::*;

use super::*;

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
        log::trace!("running");

        let mut completed = vec![];
        let mut requests = vec![];

        for (entity, nav, _) in
            (&*data.entities, &mut data.navigation_move_to, !&data.action).join()
        {
            match nav.next() {
                Some(action) => requests.push((entity, ActionRequest(action))),
                None => completed.push(entity),
            }
        }

        let requests_storage = &mut data.action_request;
        for (entity, action) in requests {
            log::debug!(
                "{:?} navigation requesting next action {:?}",
                entity,
                action,
            );
            requests_storage.insert(entity, action).unwrap();
        }

        let navigation = &mut data.navigation;
        let navigation_move_to_storage = &mut data.navigation_move_to;
        for entity in completed {
            log::debug!("{:?} navigation complete", entity);

            navigation.remove(entity).unwrap();
            navigation_move_to_storage.remove(entity).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;

    #[test]
    fn test_navigation_move_to_system_should_complete_when_path_is_empty() {
        let (world, (entity, _target)) = test_system(NavigationSystem, |world| {
            let target_id = world.create_entity().build();

            let entity = world
                .create_entity()
                .with(Navigation::MoveTo)
                .with(NavigationMoveTo {
                    target_id: target_id,
                    plan: NavigationPlan {
                        path: Default::default(),
                    },
                })
                .build();

            (entity, target_id)
        });

        let nav_storage = world.read_component::<Navigation>();
        assert!(nav_storage.get(entity).is_none());

        let nav_move_to_storage = world.read_component::<NavigationMoveTo>();
        assert!(nav_move_to_storage.get(entity).is_none());
    }
}
