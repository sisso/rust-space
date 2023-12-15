// use crate::game::actions::ActionProgress;
// use crate::game::utils::TotalTime;
//
// use bevy_ecs::prelude::*;
// use std::borrow::BorrowMut;
//
// pub struct ActionProgressSystem;
//
//
//     pub fn system_progress(&mut self, mut data: ActionProgressData) {
//         log::trace!("running");
//
//         let now = data.total_time.clone();
//         let mut completed = vec![];
//
//         for (entity, action) in (&*data.entities, &data.action_progress).join() {
//             if now.is_after(action.complete_time) {
//                 log::debug!("{:?} complete action progress", entity);
//                 completed.push(entity);
//             }
//         }
//
//         let storage = data.action_progress.borrow_mut();
//         for e in completed {
//             storage.remove(e);
//         }
//     }
//
// #[cfg(test)]
// mod test {
//     use crate::game::actions::action_progress_system::ActionProgressSystem;
//     use crate::game::actions::ActionProgress;
//     use crate::game::utils::TotalTime;
//     use crate::test::test_system;
//
//     #[test]
//     fn action_progress_should_be_ignored_if_not_complete() {
//         let (world, entity) = test_system(ActionProgressSystem, |world| {
//             world.insert_resource(TotalTime(1.0));
//             world
//                 .spawn_empty()
//                 .insert(ActionProgress {
//                     complete_time: TotalTime(2.0),
//                 })
//                 .id()
//         });
//
//         assert!(world.get::<ActionProgress>(entity).is_some());
//     }
//
//     #[test]
//     fn action_progress_should_be_removed_when_complete() {
//         let (world, entity) = test_system(ActionProgressSystem, |world| {
//             world.insert_resource(TotalTime(1.0));
//             world
//                 .spawn_empty()
//                 .insert(ActionProgress {
//                     complete_time: TotalTime(0.5),
//                 })
//                 .id()
//         });
//
//         assert!(world.get::<ActionProgress>(entity).is_none());
//     }
// }
