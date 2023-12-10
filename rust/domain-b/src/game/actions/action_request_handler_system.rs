use bevy_ecs::prelude::*;

use super::*;

use std::borrow::BorrowMut;

pub struct ActionRequestHandlerSystem;

#[derive(SystemData)]
pub struct ActionRequestHandlerData<'a> {
    entities: Entities<'a>,
    requests: WriteStorage<'a, ActionRequest>,
    actions: WriteStorage<'a, ActionActive>,
    actions_undock: WriteStorage<'a, ActionUndock>,
    actions_dock: WriteStorage<'a, ActionDock>,
    actions_move_to: WriteStorage<'a, ActionMoveTo>,
    actions_jump: WriteStorage<'a, ActionJump>,
    actions_extract: WriteStorage<'a, ActionExtract>,
    actions_generic: WriteStorage<'a, ActionGeneric>,
}

impl<'a> System<'a> for ActionRequestHandlerSystem {
    type SystemData = ActionRequestHandlerData<'a>;

    fn run(&mut self, mut data: ActionRequestHandlerData) {
        log::trace!("running");

        for (obj_id, request) in (&*data.entities, data.requests.drain()).join() {
            // used to move to ActionActive before remove from storage
            let action: Action = request.0;

            // remove any action
            let _ = data.actions_undock.remove(obj_id);
            let _ = data.actions_dock.remove(obj_id);
            let _ = data.actions_dock.remove(obj_id);
            let _ = data.actions_move_to.remove(obj_id);
            let _ = data.actions_jump.remove(obj_id);
            let _ = data.actions_extract.remove(obj_id);
            let _ = data.actions_generic.remove(obj_id);

            // update action
            match action {
                Action::Undock {} => {
                    data.actions_undock.insert(obj_id, ActionUndock).unwrap();
                }
                Action::Jump { .. } => {
                    data.actions_jump.insert(obj_id, ActionJump::new()).unwrap();
                }
                Action::Dock { .. } => {
                    data.actions_dock.insert(obj_id, ActionDock).unwrap();
                }
                Action::MoveTo { .. } => {
                    data.actions_move_to.insert(obj_id, ActionMoveTo).unwrap();
                }
                Action::MoveToTargetPos { .. } => {
                    data.actions_move_to.insert(obj_id, ActionMoveTo).unwrap();
                }
                Action::Extract { .. } => {
                    data.actions_extract
                        .insert(obj_id, ActionExtract::default())
                        .unwrap();
                }
                _ => {
                    data.actions_generic
                        .insert(obj_id, ActionGeneric {})
                        .unwrap();
                }
            }

            data.actions
                .borrow_mut()
                .insert(obj_id, ActionActive(action))
                .unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_system;

    #[test]
    fn test_action_request() {
        let (world, entity) = test_system(ActionRequestHandlerSystem, |world| {
            let entity = world
                .spawn_empty()
                .insert(ActionRequest(Action::Undock))
                .id();

            entity
        });

        let action_storage = world.read_component::<ActionActive>();
        let action = action_storage.get(entity).unwrap();
        match action.get_action() {
            Action::Undock => {}
            _ => panic!(),
        }

        let action_undock_storage = world.read_component::<ActionUndock>();
        assert!(action_undock_storage.get(entity).is_some());

        let action_request_storage = world.read_component::<ActionRequest>();
        assert!(action_request_storage.get(entity).is_none());
    }
}
