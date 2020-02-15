use specs::prelude::*;

use super::super::locations::*;
use super::*;
use crate::game::actions::*;
use std::borrow::{Borrow, BorrowMut};

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
}

impl<'a> System<'a> for ActionRequestHandlerSystem {
    type SystemData = ActionRequestHandlerData<'a>;

    fn run(&mut self, mut data: ActionRequestHandlerData) {
        trace!("running");

        let mut processed = vec![];

        for (entity, request) in (&*data.entities, &data.requests).join() {
            processed.push(entity);

            // used to move to ActionActive before remove from storage
            let action: Action = request.get_action().clone();

            let _ = data.actions_undock.borrow_mut().remove(entity);
            let _ = data.actions_dock.borrow_mut().remove(entity);
            let _ = data.actions_dock.borrow_mut().remove(entity);
            let _ = data.actions_move_to.borrow_mut().remove(entity);
            let _ = data.actions_jump.borrow_mut().remove(entity);
            let _ = data.actions_extract.borrow_mut().remove(entity);

            match action {
                Action::Undock {} => {
                    data.actions_undock
                        .borrow_mut()
                        .insert(entity, ActionUndock)
                        .unwrap();
                }
                Action::Jump { .. } => {
                    data.actions_jump
                        .borrow_mut()
                        .insert(entity, ActionJump::new())
                        .unwrap();
                }
                Action::Dock { .. } => {
                    data.actions_dock
                        .borrow_mut()
                        .insert(entity, ActionDock)
                        .unwrap();
                }
                Action::MoveTo { .. } => {
                    data.actions_move_to
                        .borrow_mut()
                        .insert(entity, ActionMoveTo)
                        .unwrap();
                }
                Action::Extract { .. } => {
                    data.actions_extract
                        .borrow_mut()
                        .insert(entity, ActionExtract)
                        .unwrap();
                },
                other => panic!("not implemented for {:?}", other),
            }

            data.actions
                .borrow_mut()
                .insert(entity, ActionActive(action))
                .unwrap();
        }

        let requests_storage = data.requests.borrow_mut();
        for entity in processed {
            requests_storage.remove(entity).unwrap();
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
                .create_entity()
                .with(ActionRequest(Action::Undock))
                .build();

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
