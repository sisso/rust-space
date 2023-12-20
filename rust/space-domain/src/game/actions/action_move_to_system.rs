use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{CommandSendEvent, EventKind, GEvent};

pub fn system_move(
    delta_time: Res<DeltaTime>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActionActive, &Moveable), With<ActionMoveTo>>,
    mut query_locations: Query<(Entity, Option<&mut LocationSpace>, Option<&LocationDocked>)>,
) {
    log::trace!("running");

    // refresh last position from a target id
    for (obj_id, mut action, _) in &mut query {
        match action.get_action_mut() {
            Action::MoveToTargetPos {
                target_id,
                last_position,
            } => {
                match Locations::resolve_space_position(&query_locations.to_readonly(), *target_id)
                {
                    Some(location) => *last_position = Some(location.pos),
                    None => {
                        log::warn!("{:?} target not found", target_id);
                        commands
                            .get_entity(obj_id)
                            .unwrap()
                            .remove::<ActionMoveTo>()
                            .remove::<ActionActive>();
                        continue;
                    }
                }
            }
            _ => {}
        };
    }

    // update movement
    for (obj_id, action, moveable) in &query {
        let target_pos = match action.get_action() {
            Action::MoveTo { pos } => *pos,
            Action::MoveToTargetPos { last_position, .. } if last_position.is_some() => {
                last_position.unwrap()
            }
            _ => continue,
        };

        // get current location
        let mut loc = match query_locations
            .get_mut(obj_id)
            .expect("obj has no locations")
        {
            (_, Some(space_loc), _) => space_loc,
            _ => {
                log::warn!(
                    "{:?} fail to move, it has no space location, skipping",
                    obj_id
                );
                continue;
            }
        };

        // compute movement
        let speed = moveable.speed.as_f32();
        let max_distance = speed * delta_time.as_f32();

        let (new_pos, complete) =
            crate::game::utils::move_towards(loc.pos, target_pos, max_distance);
        if complete {
            // if current move distance is bigger that distance to arrive, move to the position
            log::debug!("{:?} move complete", obj_id);
            loc.pos = target_pos;
            commands.add(CommandSendEvent::from(GEvent::new(obj_id, EventKind::Move)));
            commands
                .get_entity(obj_id)
                .unwrap()
                .remove::<ActionMoveTo>()
                .remove::<ActionActive>();
        } else {
            // move forward
            log::trace!(
                "{:?} moving to {:?}, new position is {:?}",
                obj_id,
                target_pos,
                new_pos,
            );
            loc.pos = new_pos;
            commands.add(CommandSendEvent::from(GEvent::new(obj_id, EventKind::Move)));
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::game::utils::{Position, Speed};
    use crate::test::{assert_v2, test_system, TestSystemRunner};

    #[test]
    fn test_move_to_system_should_move_to_target() {
        let (world, entity) = test_system(system_move, |world| {
            let sector_0 = world.spawn_empty().id();

            world.insert_resource(DeltaTime(1.0));

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .insert(ActionMoveTo::default())
                .insert(LocationSpace {
                    pos: Position::ZERO,
                    sector_id: sector_0,
                })
                .insert(Moveable { speed: Speed(1.0) })
                .id();

            entity
        });

        assert!(world.get::<ActionActive>(entity).is_some());
        assert!(world.get::<ActionMoveTo>(entity).is_some());
        let location = world.get::<LocationSpace>(entity).unwrap();
        assert_v2(location.pos, Position::new(1.0, 0.0));
    }

    #[test]
    fn test_move_to_system_should_stop_on_arrival() {
        let (world, entity) = test_system(system_move, |world| {
            let sector_0 = world.spawn_empty().id();

            world.insert_resource(DeltaTime(1.0));

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::MoveTo {
                    pos: Position::new(2.0, 0.0),
                }))
                .insert(ActionMoveTo::default())
                .insert(LocationSpace {
                    pos: Position::new(1.0, 0.0),
                    sector_id: sector_0,
                })
                .insert(Moveable { speed: Speed(1.5) })
                .id();

            entity
        });

        assert!(world.get::<ActionActive>(entity).is_none());
        assert!(world.get::<ActionMoveTo>(entity).is_none());
        let location = world.get::<LocationSpace>(entity).unwrap();
        assert_v2(location.pos, Position::new(2.0, 0.0));
    }

    #[test]
    fn test_move_to_should_follow_target_until_hit_it() {
        // create world
        let mut ts = TestSystemRunner::new(system_move);

        // add entities
        let sector_0 = ts.world.spawn_empty().id();

        ts.world.insert_resource(DeltaTime(1.0));

        let target_id = ts
            .world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::new(2.0, 0.0),
                sector_id: sector_0,
            })
            .id();

        let entity = ts
            .world
            .spawn_empty()
            .insert(ActionActive(Action::MoveToTargetPos {
                target_id,
                last_position: None,
            }))
            .insert(ActionMoveTo::default())
            .insert(LocationSpace {
                pos: Position::new(0.0, 0.0),
                sector_id: sector_0,
            })
            .insert(Moveable { speed: Speed(1.0) })
            .id();

        // run once
        ts.tick();

        // check not finished
        assert!(ts.world.get::<ActionActive>(entity).is_some());
        assert!(ts.world.get::<ActionMoveTo>(entity).is_some());

        // move target
        {
            ts.world.entity_mut(target_id).insert(LocationSpace {
                sector_id: sector_0,
                pos: Position::new(1.5, 0.5),
            });
        }

        // run twice
        ts.tick();

        // check if finish
        assert!(ts.world.get::<ActionActive>(entity).is_none());
        assert!(ts.world.get::<ActionMoveTo>(entity).is_none());

        // check final location
        let location = ts.world.get::<LocationSpace>(entity).unwrap();
        assert_v2(location.pos, Position::new(1.5, 0.5));
    }
}
