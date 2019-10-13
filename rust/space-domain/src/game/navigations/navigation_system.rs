use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::actions::*;

///
/// Execute actions for each NavigationMoveto without Action
///
///
///
pub struct NavigationSystem;

#[derive(SystemData)]
pub struct NavigationData<'a> {
    entities: Entities<'a>,
    navigation_move_to: WriteStorage<'a, NavigationMoveTo>,
    action: ReadStorage<'a, Action>,
    action_request: WriteStorage<'a, ActionRequest>,
}

impl<'a> System<'a> for NavigationSystem {
    type SystemData = NavigationData<'a>;

    fn run(&mut self, mut data: NavigationData) {

        let mut completed = vec![];
        let mut requests = vec![];

        for (entity, nav, _) in (&*data.entities, &mut data.navigation_move_to, !&data.action).join() {
            match nav.next() {
                Some(action_request) => requests.push((entity, action_request)),
                None => completed.push(entity),
            }
        }

        let requests_storage = data.action_request.borrow_mut();
        for (entity, action) in requests {
            let _ = requests_storage.insert(entity, action).unwrap();
        }

        let navigation_move_to_storage = data.navigation_move_to.borrow_mut();
        for entity in completed {
            let _ = navigation_move_to_storage.remove(entity).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stuff() {
        let mut world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(NavigationSystem, "test", &[])
            .build();
        dispatcher.setup(&mut world);

        let entity = world.create_entity()
            .build();

        dispatcher.dispatch(&world);
        world.maintain();

        let nav_storage = world.read_component::<Navigation>();
        let nav = nav_storage.get(entity).unwrap();
        assert_eq!(nav.clone(), Navigation::MoveTo);
    }
}
