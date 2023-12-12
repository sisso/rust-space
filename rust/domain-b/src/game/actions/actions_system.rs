use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{CommandSendEvent, EventKind, GEvent};

use crate::game::utils::Speed;
use commons::math;

pub fn system_actions(
    total_time: Res<TotalTime>,
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &ActionActive,
            Option<&LocationSpace>,
            Option<&LocationDocked>,
        ),
        With<ActionGeneric>,
    >,
    query_locations: Query<&LocationSpace>,
) {
    log::trace!("running");

    let total_time = *total_time;

    for (obj_id, action, maybe_space, maybe_docked) in &query {
        match action.get_action() {
            Action::Orbit { target_id } => {
                if maybe_docked.is_some() {
                    log::warn!(
                        "{:?} orbit action fail, can not orbit, it is currently docked",
                        obj_id
                    );
                } else {
                    match (maybe_space, query_locations.get(*target_id).ok()) {
                        (Some(obj_loc), Some(target_loc))
                            if obj_loc.sector_id == target_loc.sector_id =>
                        {
                            let distance = target_loc.pos.distance(obj_loc.pos);
                            // compute angle from parent, not from the orbiting object
                            let angle = math::angle_vector(obj_loc.pos - target_loc.pos);
                            let speed = Speed(1.0);

                            let orbit = LocationOrbit {
                                parent_id: *target_id,
                                distance,
                                start_time: total_time,
                                start_angle: angle,
                                speed: speed,
                            };

                            log::trace!("{:?} setting orbit {:?}", obj_id, orbit);

                            commands.entity(obj_id).insert(orbit);
                            commands.add(CommandSendEvent::from(GEvent {
                                id: obj_id,
                                kind: EventKind::Orbit,
                            }));
                        }
                        _ => {
                            log::warn!("{:?} orbit action fail, self or target are not in space or in different sectors", obj_id);
                        }
                    }
                }
            }
            Action::Deorbit => {
                commands.entity(obj_id).remove::<LocationOrbit>();
                commands.add(CommandSendEvent::from(GEvent {
                    id: obj_id,
                    kind: EventKind::Deorbit,
                }));
            }
            _ => continue,
        };

        commands
            .entity(obj_id)
            .remove::<ActionActive>()
            .remove::<ActionGeneric>();
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::test::test_system;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_get_into_orbit() {
        let (world, (obj_id, asteroid_id)) = test_system(system_actions, |world| {
            let sector_id = world.spawn_empty().id();

            world.insert_resource(TotalTime(2.0));

            let asteroid_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id,
                })
                .id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .insert(ActionGeneric {})
                .insert(LocationSpace {
                    pos: P2::X,
                    sector_id,
                })
                .id();

            (entity, asteroid_id)
        });

        // check task is done
        assert!(world.get::<ActionActive>(obj_id).is_none());
        assert!(world.get::<ActionGeneric>(obj_id).is_none());

        let orbit = world
            .get::<LocationOrbit>(obj_id)
            .cloned()
            .expect("orbit not found");

        assert_eq!(asteroid_id, orbit.parent_id);
        assert_abs_diff_eq!(1.0, orbit.distance);
        assert_abs_diff_eq!(0.0, orbit.start_angle);
        assert_abs_diff_eq!(2.0, orbit.start_time.as_f64());
    }

    #[test]
    fn test_do_not_orbit_if_different_sector() {
        let (world, (obj_id,)) = test_system(system_actions, |world| {
            let sector_id_0 = world.spawn_empty().id();
            let sector_id_1 = world.spawn_empty().id();

            world.insert_resource(TotalTime(2.0));

            let asteroid_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id_0,
                })
                .id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .insert(ActionGeneric {})
                .insert(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id_1,
                })
                .id();

            (entity,)
        });

        // check task is done
        assert!(world.get::<ActionActive>(obj_id).is_none());
        assert!(world.get::<ActionGeneric>(obj_id).is_none());
        assert!(world.get::<LocationOrbit>(obj_id).is_none());
    }

    #[test]
    fn test_do_not_orbit_if_is_docked() {
        let (world, (obj_id,)) = test_system(system_actions, |world| {
            let sector_id = world.spawn_empty().id();

            world.insert_resource(TotalTime(2.0));

            let asteroid_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .id();

            let station_id = world.spawn_empty().id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .insert(ActionGeneric {})
                .insert(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .insert(LocationDocked {
                    parent_id: station_id,
                })
                .id();

            (entity,)
        });

        // check task is done
        assert!(world.get::<ActionActive>(obj_id).is_none());
        assert!(world.get::<ActionGeneric>(obj_id).is_none());
        assert!(world.get::<LocationOrbit>(obj_id).is_none());
    }

    #[test]
    fn test_do_deorbit() {
        let (world, (obj_id,)) = test_system(system_actions, |world| {
            let sector_id = world.spawn_empty().id();

            world.insert_resource(TotalTime(2.0));

            let asteroid_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Deorbit {}))
                .insert(ActionGeneric {})
                .insert(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .insert(LocationOrbit::new(asteroid_id))
                .id();

            (entity,)
        });

        // check task is done
        assert!(world.get::<ActionActive>(obj_id).is_none());
        assert!(world.get::<ActionGeneric>(obj_id).is_none());
        assert!(world.get::<LocationOrbit>(obj_id).is_none());
    }

    #[test]
    fn test_do_deorbit_if_not_orbiting() {
        let (world, (obj_id, _asteroid_id)) = test_system(system_actions, |world| {
            let sector_id = world.spawn_empty().id();

            world.insert_resource(TotalTime(2.0));

            let asteroid_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Deorbit {}))
                .insert(ActionGeneric {})
                .insert(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .id();

            (entity, asteroid_id)
        });

        // check task is done
        assert!(world.get::<ActionActive>(obj_id).is_none());
        assert!(world.get::<ActionGeneric>(obj_id).is_none());
        assert!(world.get::<LocationOrbit>(obj_id).is_none());
    }
}
