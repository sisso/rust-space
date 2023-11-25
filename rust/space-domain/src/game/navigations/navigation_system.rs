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
    action: ReadStorage<'a, ActionActive>,
    action_request: WriteStorage<'a, ActionRequest>,
}

impl<'a> System<'a> for NavigationSystem {
    type SystemData = NavigationData<'a>;

    fn run(&mut self, mut data: NavigationData) {
        log::trace!("running");

        let mut completed = vec![];

        // for each navigation without active action
        for (entity, nav, _) in (&*data.entities, &mut data.navigation, !&data.action).join() {
            // pop next action form path
            match nav.plan.path.pop_front() {
                Some(action) => {
                    log::debug!(
                        "{:?} navigation requesting next action {:?}",
                        entity,
                        action,
                    );
                    data.action_request
                        .insert(entity, ActionRequest(action))
                        .unwrap();
                }
                None => completed.push(entity),
            }
        }

        for entity in completed {
            log::debug!("{:?} navigation complete", entity);
            data.navigation.remove(entity).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;
    use crate::utils::V2;

    #[test]
    fn test_navigation_move_to_system_should_complete_when_path_is_empty() {
        let (world, (entity, _target)) = test_system(NavigationSystem, |world| {
            let target_id = world.create_entity().build();

            let entity = world
                .create_entity()
                .with(Navigation {
                    request: NavRequest::MoveToPos {
                        sector_id: target_id,
                        pos: V2::ZERO,
                    },
                    plan: NavigationPlan {
                        path: Default::default(),
                    },
                })
                .build();

            (entity, target_id)
        });

        let nav_storage = world.read_component::<Navigation>();
        assert!(nav_storage.get(entity).is_none());

        let nav_move_to_storage = world.read_component::<Navigation>();
        assert!(nav_move_to_storage.get(entity).is_none());
    }
}
