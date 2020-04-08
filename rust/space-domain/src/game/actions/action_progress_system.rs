use crate::game::actions::ActionProgress;
use crate::utils::TotalTime;
use specs::prelude::*;
use std::borrow::BorrowMut;

pub struct ActionProgressSystem;

#[derive(SystemData)]
pub struct ActionProgressData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    action_progress: WriteStorage<'a, ActionProgress>,
}

impl<'a> System<'a> for ActionProgressSystem {
    type SystemData = ActionProgressData<'a>;

    fn run(&mut self, mut data: ActionProgressData) {
        trace!("running");

        let now = data.total_time.clone();
        let mut completed = vec![];

        for (entity, action) in (&*data.entities, &data.action_progress).join() {
            if now.is_after(action.complete_time) {
                debug!("{:?} complete action progress", entity);
                completed.push(entity);
            }
        }

        let storage = data.action_progress.borrow_mut();
        for e in completed {
            storage.remove(e);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::actions::action_progress_system::ActionProgressSystem;
    use crate::game::actions::ActionProgress;
    use crate::test::test_system;
    use crate::utils::TotalTime;
    use specs::{Builder, WorldExt};

    #[test]
    fn action_progress_should_be_ignored_if_not_complete() {
        let (world, entity) = test_system(ActionProgressSystem, |world| {
            world.insert(TotalTime(1.0));
            world
                .create_entity()
                .with(ActionProgress {
                    complete_time: TotalTime(2.0),
                })
                .build()
        });

        assert!(world.read_storage::<ActionProgress>().get(entity).is_some());
    }

    #[test]
    fn action_progress_should_be_removed_when_complete() {
        let (world, entity) = test_system(ActionProgressSystem, |world| {
            world.insert(TotalTime(1.0));
            world
                .create_entity()
                .with(ActionProgress {
                    complete_time: TotalTime(0.5),
                })
                .build()
        });

        assert!(world.read_storage::<ActionProgress>().get(entity).is_none());
    }
}
