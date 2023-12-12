use bevy_ecs::prelude::*;

use super::*;

pub fn system_action_request(
    mut commands: Commands,
    query: Query<(Entity, &ActionRequest), Without<ActionActive>>,
) {
    log::trace!("running");

    for (obj_id, request) in &query {
        // used to move to ActionActive before remove from storage
        let action = request.0.clone();

        let mut entity = commands.get_entity(obj_id).unwrap();

        // remove leaked action component
        entity
            .remove::<ActionUndock>()
            .remove::<ActionDock>()
            .remove::<ActionMoveTo>()
            .remove::<ActionJump>()
            .remove::<ActionExtract>()
            .remove::<ActionGeneric>();

        // update action
        let _ = match action {
            Action::Undock {} => entity.insert(ActionUndock),
            Action::Jump { .. } => entity.insert(ActionJump::new()),
            Action::Dock { .. } => entity.insert(ActionDock),
            Action::MoveTo { .. } => entity.insert(ActionMoveTo),
            Action::MoveToTargetPos { .. } => entity.insert(ActionMoveTo),
            Action::Extract { .. } => entity.insert(ActionExtract::default()),
            _ => entity.insert(ActionGeneric {}),
        };

        log::debug!("{:?} updating active action {:?}", obj_id, action);
        entity
            .insert(ActionActive(action))
            .remove::<ActionRequest>();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;

    #[test]
    fn test_action_request() {
        let (world, entity) = test_system(system_action_request, |world| {
            let entity = world
                .spawn_empty()
                .insert(ActionRequest(Action::Undock))
                .id();

            entity
        });

        let action = world.get::<ActionActive>(entity).unwrap();
        match action.get_action() {
            Action::Undock => {}
            _ => panic!(),
        }

        assert!(world.get::<ActionUndock>(entity).is_some());
        assert!(world.get::<ActionRequest>(entity).is_none());
    }
}
