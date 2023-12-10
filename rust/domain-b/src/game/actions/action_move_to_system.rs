use bevy_ecs::prelude::*;

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
    locations: WriteStorage<'a, LocationSpace>,
    locations_docked: WriteStorage<'a, LocationDocked>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for ActionMoveToSystem {
    type SystemData = ActionMoveToData<'a>;

    fn run(&mut self, mut data: ActionMoveToData) {
        log::trace!("running");

        let mut completed = vec![];

        let delta_time = &data.delta_time;

        // refresh last position from a target id
        for (entity, action) in (&*data.entities, &mut data.actions).join() {
            match action.get_action_mut() {
                Action::MoveToTargetPos {
                    target_id,
                    last_position,
                } => match Locations::resolve_space_position(
                    &data.locations,
                    &data.locations_docked,
                    *target_id,
                ) {
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
        for (entity, moveable, action, _, loc) in (
            &*data.entities,
            &data.moveable,
            &data.actions,
            &data.action_move_to,
            &mut data.locations,
        )
            .join()
        {
            let target_pos = match action.get_action() {
                Action::MoveTo { pos } => *pos,
                Action::MoveToTargetPos { last_position, .. } if last_position.is_some() => {
                    last_position.unwrap()
                }
                _ => continue,
            };

            // compute movement
            let speed = moveable.speed.as_f32();
            let max_distance = speed * delta_time.as_f32();

            let (new_pos, complete) =
                crate::game::utils::move_towards(loc.pos, target_pos, max_distance);
            if complete {
                // if current move distance is bigger that distance to arrive, move to the position
                log::debug!("{:?} move complete", entity);
                loc.pos = target_pos;
                data.events.push(Event::new(entity, EventKind::Move));
                completed.push(entity);
            } else {
                // move forward
                log::trace!(
                    "{:?} moving to {:?}, new position is {:?}",
                    entity,
                    target_pos,
                    new_pos,
                );
                loc.pos = new_pos;
                data.events.push(Event::new(entity, EventKind::Move));
            }
        }

        // remove completed action
        for entity in completed {
            (&mut data.actions).remove(entity).unwrap();
            (&mut data.action_move_to).remove(entity).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::game::utils::{Position, Speed};
    use crate::test::{assert_v2, test_system};

    #[test]
    fn test_move_to_system_should_move_to_target() {
        let (world, entity) = test_system(ActionMoveToSystem, |world| {
            let sector_0 = world.spawn_empty().id();

            world.insert(DeltaTime(1.0));

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .insert(ActionMoveTo)
                .insert(LocationSpace {
                    pos: Position::ZERO,
                    sector_id: sector_0,
                })
                .insert(Moveable { speed: Speed(1.0) })
                .id();

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
            let sector_0 = world.spawn_empty().id();

            world.insert(DeltaTime(1.0));

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .insert(ActionMoveTo)
                .insert(LocationSpace {
                    pos: Position::new(1.0, 0.0),
                    sector_id: sector_0,
                })
                .insert(Moveable { speed: Speed(1.5) })
                .id();

            entity
        });

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, Position::new(2.0, 0.0));
    }

    #[test]
    fn test_move_to_should_follow_target_until_hit_it() {
        // create world
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .insert(ActionMoveToSystem, "test", &[])
            .id();
        dispatcher.setup(&mut world);

        // add entities
        let sector_0 = world.spawn_empty().id();

        world.insert(DeltaTime(1.0));

        let target = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::new(2.0, 0.0),
                sector_id: sector_0,
            })
            .id();

        let entity = world
            .spawn_empty()
            .insert(ActionActive(Action::MoveToTargetPos {
                target_id: target,
                last_position: None,
            }))
            .insert(ActionMoveTo)
            .insert(LocationSpace {
                pos: Position::new(0.0, 0.0),
                sector_id: sector_0,
            })
            .insert(Moveable { speed: Speed(1.0) })
            .id();

        // run once
        dispatcher.dispatch(&world);
        world.maintain();

        // check not finished
        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_some());

        // move target
        {
            let l = &mut world.write_storage::<LocationSpace>();
            l.insert(
                target,
                LocationSpace {
                    sector_id: sector_0,
                    pos: Position::new(1.5, 0.5),
                },
            )
            .unwrap();
        }

        // run twice
        dispatcher.dispatch(&world);
        world.maintain();

        // check if finish
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionMoveTo>().get(entity).is_none());

        // check final location
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, Position::new(1.5, 0.5));
    }
}
