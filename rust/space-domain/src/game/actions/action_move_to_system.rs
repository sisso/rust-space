use specs::prelude::*;

use super::super::locations::*;
use super::*;
use crate::game::actions::*;
use std::borrow::{Borrow, BorrowMut};

pub struct ActionMoveToSystem;

#[derive(SystemData)]
pub struct ActionMoveToData<'a> {
    entities: Entities<'a>,
    delta_time: Read<'a, DeltaTime>,
    moveable: ReadStorage<'a, Moveable>,
    actions: WriteStorage<'a, ActionActive>,
    action_move_to: WriteStorage<'a, ActionMoveTo>,
    location_space: WriteStorage<'a, LocationSpace>,
}

impl<'a> System<'a> for ActionMoveToSystem {
    type SystemData = ActionMoveToData<'a>;

    fn run(&mut self, mut data: ActionMoveToData) {
        trace!("running");

        let mut completed = vec![];
        let delta_time = data.delta_time.borrow();

        for (entity, moveable, action, _, position) in (
            &data.entities,
            &data.moveable,
            &data.actions,
            &data.action_move_to,
            &mut data.location_space,
        )
            .join()
        {
            let to = match action.get_action() {
                Action::MoveTo { pos } => pos.clone(),
                _ => continue,
            };

            let delta = to.sub(&position.pos);
            // delta == zero can cause length sqr NaN
            let length_sqr = delta.length_sqr();
            let speed = moveable.speed.as_f32();
            let max_distance = speed * delta_time.as_f32();
            let norm = delta.div(length_sqr.sqrt());
            let mov = norm.mult(max_distance);

            // if current move distance is bigger that distance to arrive, move to the position
            if length_sqr.is_nan() || length_sqr <= max_distance {
                debug!("{:?} move complete", entity);
                position.pos = to;
                completed.push(entity.clone());
            } else {
                let new_pos = position.pos.add(&mov);
                trace!(
                    "{:?} moving to {:?}, new position is {:?}",
                    entity,
                    to,
                    new_pos
                );
                position.pos = new_pos;
            }
        }

        for entity in completed {
            let _ = data.actions.borrow_mut().remove(entity);
            let _ = data.action_move_to.borrow_mut().remove(entity);
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::test::{assert_v2, test_system};
    use crate::utils::Speed;

    #[test]
    fn test_move_to_system_should_move_to_target() {
        let (world, entity) = test_system(ActionMoveToSystem, |world| {
            world.insert(DeltaTime(1.0));

            let entity = world
                .create_entity()
                .with(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .with(ActionMoveTo)
                .with(LocationSpace {
                    pos: Position::new(0.0, 0.0),
                })
                .with(Moveable { speed: Speed(1.0) })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_some());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, Position::new(1.0, 0.0));
    }

    #[test]
    fn test_move_to_system_should_stop_on_arrival() {
        let (world, entity) = test_system(ActionMoveToSystem, |world| {
            world.insert(DeltaTime(1.0));

            let entity = world
                .create_entity()
                .with(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .with(ActionMoveTo)
                .with(LocationSpace {
                    pos: Position::new(1.0, 0.0),
                })
                .with(Moveable { speed: Speed(1.5) })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, Position::new(2.0, 0.0));
    }
}
