use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

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

        // refresh any position from a target id
        for (entity, action) in (&*data.entities, &mut data.actions).join() {
            match action.get_action_mut() {
                Action::MoveToTargetPos {
                    target_id,
                    last_position,
                } => match Locations::resolve_space_position(&data.locations, *target_id) {
                    Some(location) => *last_position = Some(location.pos),
                    None => {
                        log::warn!("{:?} target not found", target_id);
                        completed.push(entity);
                        continue;
                    }
                },
                _ => {}
            };
        }

        // update movement
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
                Action::MoveTo { pos } => *pos,
                Action::MoveToTargetPos { last_position, .. } if last_position.is_some() => {
                    last_position.unwrap()
                }
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

            let (new_pos, complete) = crate::utils::move_towards(pos, to, max_distance);

            // if current move distance is bigger that distance to arrive, move to the position
            if complete {
                log::debug!("{:?} move complete", entity);
                location.set_pos(to).unwrap();
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
    use crate::test::{assert_v2, init_log, test_system};
    use crate::utils::{Position, Speed};

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
                    pos: Position::ZERO,
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

    #[test]
    fn test_move_to_should_follow_target_until_hit_it() {
        init_log();

        // create world
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(ActionMoveToSystem, "test", &[])
            .build();
        dispatcher.setup(&mut world);

        // add entities
        let sector_0 = world.create_entity().build();

        world.insert(DeltaTime(1.0));

        let target = world
            .create_entity()
            .with(Location::Space {
                pos: Position::new(2.0, 0.0),
                sector_id: sector_0,
            })
            .build();

        let entity = world
            .create_entity()
            .with(ActionActive(Action::MoveToTargetPos {
                target_id: target,
                last_position: None,
            }))
            .with(ActionMoveTo)
            .with(Location::Space {
                pos: Position::new(0.0, 0.0),
                sector_id: sector_0,
            })
            .with(Moveable { speed: Speed(1.0) })
            .build();

        // run once
        dispatcher.dispatch(&world);
        world.maintain();

        // check not finished
        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_some());

        // move target
        {
            let mut l = &mut world.write_storage::<Location>();
            l.insert(
                target,
                Location::Space {
                    sector_id: sector_0,
                    pos: Position::new(1.5, 0.5),
                },
            );
        }

        // run twice
        dispatcher.dispatch(&world);
        world.maintain();

        // check if finish
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_none());

        // check final location
        let storage = world.read_storage::<Location>();
        let location = storage.get(entity).unwrap().as_space().unwrap();
        assert_v2(location.pos, Position::new(1.5, 0.5));
    }
}
