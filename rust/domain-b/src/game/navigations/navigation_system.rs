use bevy_ecs::prelude::*;

use super::*;

///
/// Execute actions for each NavigationMoveto without Action
///
pub fn system_navigation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Navigation), Without<ActionActive>>,
) {
    log::trace!("running");

    let mut completed = vec![];

    // for each navigation without active action
    for (obj_id, mut nav) in &mut query {
        // pop next action form path
        match nav.plan.path.pop_front() {
            Some(action) => {
                log::debug!(
                    "{:?} navigation requesting next action {:?}",
                    obj_id,
                    action,
                );

                commands.entity(obj_id).insert(ActionRequest(action));
            }
            None => {
                log::debug!("{:?} navigation complete", obj_id);
                commands.entity(obj_id).remove::<Navigation>();
            }
        }
    }

    for obj_id in completed {
        log::debug!("{:?} navigation complete", obj_id);
        commands
            .get_entity(obj_id)
            .expect("obj not found")
            .remove::<Navigation>();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::utils::V2;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_navigation_move_to_system_should_complete_when_path_is_empty() {
        let mut world = World::new();

        let target_id = world.spawn_empty().id();

        let obj_id = world
            .spawn_empty()
            .insert(Navigation {
                request: NavRequest::MoveToPos {
                    sector_id: target_id,
                    pos: V2::ZERO,
                },
                plan: NavigationPlan {
                    path: Default::default(),
                },
            })
            .id();

        world.run_system_once(system_navigation);

        let e = world.get_entity(obj_id).unwrap();
        assert!(e.get::<Navigation>().is_none());
        assert!(e.get::<ActionRequest>().is_none());
    }
}
