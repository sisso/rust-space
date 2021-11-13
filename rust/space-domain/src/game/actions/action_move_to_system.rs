use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use crate::utils::V2;


pub struct ActionMoveToSystem;

#[derive(SystemData)]
pub struct ActionMoveToData<'a> {
    entities: Entities<'a>,
    delta_time: Read<'a, DeltaTime>,
    moveable: ReadStorage<'a, Moveable>,
    actions: WriteStorage<'a, ActionActive>,
    action_move_to: WriteStorage<'a, ActionMoveTo>,
    locations: WriteStorage<'a, Location>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for ActionMoveToSystem {
    type SystemData = ActionMoveToData<'a>;

    fn run(&mut self, mut data: ActionMoveToData) {
        log::trace!("running");

        let mut moved = vec![];
        let mut completed = vec![];

        let delta_time = &data.delta_time;

        for (entity, moveable, action, _, location) in (
            &*data.entities,
            &data.moveable,
            &data.actions,
            &data.action_move_to,
            &mut data.locations,
        )
            .join()
        {
            let to = match action.get_action() {
                Action::MoveTo { pos } => pos,
                _ => continue,
            };

            // confirm we are dealing with space object
            let pos = match location.get_pos() {
                Some(pos) => pos,
                _ => {
                    log::warn!(
                        "{:?} can not do action move since it is not in space",
                        entity,
                    );
                    completed.push(entity);
                    continue;
                }
            };

            let speed = moveable.speed.as_f32();
            let max_distance = speed * delta_time.as_f32();

            let (new_pos, complete) = V2::move_towards(pos, to, max_distance);

            // if current move distance is bigger that distance to arrive, move to the position
            if complete {
                log::debug!("{:?} move complete", entity);
                location.set_pos(*to).unwrap();
                moved.push(entity);
                completed.push(entity);
            } else {
                log::trace!(
                    "{:?} moving to {:?}, new position is {:?}",
                    entity,
                    to,
                    new_pos,
                );
                location.set_pos(new_pos).unwrap();
                moved.push(entity);
            }
        }

        for entity in completed {
            (&mut data.actions).remove(entity).unwrap();
            (&mut data.action_move_to).remove(entity).unwrap();
        }

        let events = &mut data.events;
        for entity in moved {
            events.push(Event::new(entity, EventKind::Move));
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
            let sector_0 = world.create_entity().build();

            world.insert(DeltaTime(1.0));

            let entity = world
                .create_entity()
                .with(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .with(ActionMoveTo)
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id: sector_0,
                })
                .with(Moveable { speed: Speed(1.0) })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_some());
        let storage = world.read_storage::<Location>();
        let location = storage.get(entity).unwrap().as_space().unwrap();
        assert_v2(location.pos, Position::new(1.0, 0.0));
    }

    #[test]
    fn test_move_to_system_should_stop_on_arrival() {
        let (world, entity) = test_system(ActionMoveToSystem, |world| {
            let sector_0 = world.create_entity().build();

            world.insert(DeltaTime(1.0));

            let entity = world
                .create_entity()
                .with(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .with(ActionMoveTo)
                .with(Location::Space {
                    pos: Position::new(1.0, 0.0),
                    sector_id: sector_0,
                })
                .with(Moveable { speed: Speed(1.5) })
                .build();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_none());
        let storage = world.read_storage::<Location>();
        let location = storage.get(entity).unwrap().as_space().unwrap();
        assert_v2(location.pos, Position::new(2.0, 0.0));
    }
}
