use specs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{Event, EventKind, Events};

use crate::utils::Speed;
use commons::math;

pub struct ActionsSystem;

#[derive(SystemData)]
pub struct ActionsSystemData<'a> {
    entities: Entities<'a>,
    actions: WriteStorage<'a, ActionActive>,
    actions_generic: WriteStorage<'a, ActionGeneric>,
    location_space: WriteStorage<'a, LocationSpace>,
    location_docked: WriteStorage<'a, LocationDocked>,
    location_orbit: WriteStorage<'a, LocationOrbit>,
    events: Write<'a, Events>,
    total_time: ReadExpect<'a, TotalTime>,
}

impl<'a> System<'a> for ActionsSystem {
    type SystemData = ActionsSystemData<'a>;

    fn run(&mut self, mut data: ActionsSystemData) {
        log::trace!("running");

        let mut completed: Vec<Entity> = vec![];

        for (obj_id, action, _) in (&*data.entities, &data.actions, &data.actions_generic).join() {
            match action.get_action() {
                Action::Orbit { target_id } => {
                    if data.location_docked.contains(obj_id) {
                        log::warn!(
                            "{:?} orbit action fail, can not orbit, it is currently docked",
                            obj_id
                        );
                        completed.push(obj_id);
                        continue;
                    }

                    match (
                        data.location_space.get(obj_id),
                        data.location_space.get(*target_id),
                    ) {
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
                                start_time: *data.total_time,
                                start_angle: angle,
                                speed: speed,
                            };

                            log::trace!("{:?} setting orbit {:?}", obj_id, orbit);

                            data.location_orbit
                                .insert(obj_id, orbit)
                                .expect("fail to insert orbit");
                            completed.push(obj_id);
                            data.events.push(Event {
                                id: obj_id,
                                kind: EventKind::Orbit,
                            })
                        }
                        _ => {
                            log::warn!("{:?} orbit action fail, self or target are not in space or in different sectors", obj_id);
                            completed.push(obj_id);
                            continue;
                        }
                    }
                }
                Action::Deorbit => {
                    data.location_orbit.remove(obj_id);
                    completed.push(obj_id);
                    data.events.push(Event {
                        id: obj_id,
                        kind: EventKind::Deorbit,
                    })
                }
                _ => continue,
            };
        }

        for entity in completed {
            data.actions.remove(entity).unwrap();
            data.actions_generic.remove(entity).unwrap();
        }
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
        let (world, (obj_id, asteroid_id)) = test_system(ActionsSystem, |world| {
            let sector_id = world.create_entity().build();

            world.insert(TotalTime(2.0));

            let asteroid_id = world
                .create_entity()
                .with(LocationSpace {
                    pos: P2::ZERO,
                    sector_id,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .with(ActionGeneric {})
                .with(LocationSpace {
                    pos: P2::X,
                    sector_id,
                })
                .build();

            (entity, asteroid_id)
        });

        // check task is done
        assert!(world.read_storage::<ActionActive>().get(obj_id).is_none());
        assert!(world.read_storage::<ActionGeneric>().get(obj_id).is_none());

        let orbit = world
            .read_storage::<LocationOrbit>()
            .get(obj_id)
            .cloned()
            .expect("orbit not found");

        assert_eq!(asteroid_id, orbit.parent_id);
        assert_abs_diff_eq!(1.0, orbit.distance);
        assert_abs_diff_eq!(0.0, orbit.start_angle);
        assert_abs_diff_eq!(2.0, orbit.start_time.as_f64());
    }

    #[test]
    fn test_do_not_orbit_if_different_sector() {
        let (world, (obj_id,)) = test_system(ActionsSystem, |world| {
            let sector_id_0 = world.create_entity().build();
            let sector_id_1 = world.create_entity().build();

            world.insert(TotalTime(2.0));

            let asteroid_id = world
                .create_entity()
                .with(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id_0,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .with(ActionGeneric {})
                .with(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id_1,
                })
                .build();

            (entity,)
        });

        // check task is done
        assert!(world.read_storage::<ActionActive>().get(obj_id).is_none());
        assert!(world.read_storage::<ActionGeneric>().get(obj_id).is_none());
        assert!(world.read_storage::<LocationOrbit>().get(obj_id).is_none());
    }

    #[test]
    fn test_do_not_orbit_if_is_docked() {
        let (world, (obj_id,)) = test_system(ActionsSystem, |world| {
            let sector_id = world.create_entity().build();

            world.insert(TotalTime(2.0));

            let asteroid_id = world
                .create_entity()
                .with(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .build();

            let station_id = world.create_entity().build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Orbit {
                    target_id: asteroid_id,
                }))
                .with(ActionGeneric {})
                .with(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .with(LocationDocked {
                    parent_id: station_id,
                })
                .build();

            (entity,)
        });

        // check task is done
        assert!(world.read_storage::<ActionActive>().get(obj_id).is_none());
        assert!(world.read_storage::<ActionGeneric>().get(obj_id).is_none());
        assert!(world.read_storage::<LocationOrbit>().get(obj_id).is_none());
    }

    #[test]
    fn test_do_deorbit() {
        let (world, (obj_id,)) = test_system(ActionsSystem, |world| {
            let sector_id = world.create_entity().build();

            world.insert(TotalTime(2.0));

            let asteroid_id = world
                .create_entity()
                .with(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Deorbit {}))
                .with(ActionGeneric {})
                .with(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .with(LocationOrbit::new(asteroid_id))
                .build();

            (entity,)
        });

        // check task is done
        assert!(world.read_storage::<ActionActive>().get(obj_id).is_none());
        assert!(world.read_storage::<ActionGeneric>().get(obj_id).is_none());
        assert!(world.read_storage::<LocationOrbit>().get(obj_id).is_none());
    }

    #[test]
    fn test_do_deorbit_if_not_orbiting() {
        let (world, (obj_id, _asteroid_id)) = test_system(ActionsSystem, |world| {
            let sector_id = world.create_entity().build();

            world.insert(TotalTime(2.0));

            let asteroid_id = world
                .create_entity()
                .with(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_id,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Deorbit {}))
                .with(ActionGeneric {})
                .with(LocationSpace {
                    pos: P2::X,
                    sector_id: sector_id,
                })
                .build();

            (entity, asteroid_id)
        });

        // check task is done
        assert!(world.read_storage::<ActionActive>().get(obj_id).is_none());
        assert!(world.read_storage::<ActionGeneric>().get(obj_id).is_none());
        assert!(world.read_storage::<LocationOrbit>().get(obj_id).is_none());
    }
}
