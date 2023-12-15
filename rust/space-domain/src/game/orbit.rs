use crate::game::locations::{LocationOrbit, LocationSpace};
use crate::game::objects::ObjId;
use crate::game::utils::{Speed, TotalTime};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use commons::math;
use commons::math::{Distance, Rad, P2};
use std::collections::HashMap;

pub struct Orbits;

impl Orbits {
    pub fn update_orbits(world: &mut World) {
        world.run_system_once(system_compute_orbits);
    }
}

pub fn compute_orbit_local_pos(
    radius: Distance,
    initial_angle: Rad,
    start_time: TotalTime,
    speed: Speed,
    current_time: TotalTime,
) -> P2 {
    let base = math::TWO_PI as f64 / 10000.0f64;
    let angle = initial_angle as f64
        + (current_time.as_f64() - start_time.as_f64()) * (base * speed.0 as f64);
    math::rotate_vector_by_angle(P2::new(radius, 0.0), angle as f32).into()
}

/// try to resolve object orbit by recursive resolving orbits, if object (or any parent) has no
/// orbit, its position will be used. A cache is used to hold all computation for lookup or
/// updates.
fn compute_orbital_pos(
    cache: &mut HashMap<ObjId, LocationSpace>,
    query: &Query<(Entity, &LocationSpace, Option<&LocationOrbit>)>,
    time: TotalTime,
    id: ObjId,
) -> Result<LocationSpace, &'static str> {
    // check if is already cached
    if let Some(pos) = cache.get(&id) {
        return Ok(*pos);
    }

    let (_, loc_space, orbit) = query.get(id).map_err(|_| "obj_id not found")?;

    // get orbit
    let orbit = match orbit {
        Some(orbit) => orbit,
        None => {
            // if not orbit, update cache and return position
            cache.insert(id, loc_space.clone());
            return Ok(loc_space.clone());
        }
    };

    let parent_pos = match compute_orbital_pos(cache, query, time, orbit.parent_id) {
        Ok(pos) => pos,
        Err(err_msg) => {
            log::warn!(
                "{:?} fail to compute parent position for orbit {:?}: {}",
                id,
                orbit.parent_id,
                err_msg
            );
            return Err("parent object is docked");
        }
    };

    let local_pos = compute_orbit_local_pos(
        orbit.distance,
        orbit.start_angle,
        orbit.start_time,
        orbit.speed,
        time,
    );

    let pos = LocationSpace {
        pos: parent_pos.pos + local_pos,
        sector_id: parent_pos.sector_id,
    };
    cache.insert(id, pos);
    Ok(pos)
}

pub fn system_compute_orbits(
    total_time: Res<TotalTime>,
    mut query: Query<(Entity, &mut LocationSpace, Option<&LocationOrbit>)>,
    query_orbits: Query<Entity, With<LocationOrbit>>,
) {
    log::trace!("running");

    let mut cache: HashMap<ObjId, LocationSpace> = Default::default();
    let mut updates = vec![];
    let current_time = *total_time;
    let read_query = query.to_readonly();

    for obj_id in &query_orbits {
        let pos = match compute_orbital_pos(&mut cache, &read_query, current_time, obj_id) {
            Ok(pos) => pos,
            Err(err_msg) => {
                log::warn!("{:?} fail to generate orbiting by {}", obj_id, err_msg);
                continue;
            }
        };

        updates.push((
            obj_id,
            LocationSpace {
                pos: pos.pos,
                sector_id: pos.sector_id,
            },
        ));
    }

    for (obj_id, location) in updates {
        log::trace!("{:?} updating orbit location to {:?}", obj_id, location);
        *query.get_mut(obj_id).expect("obj_id not found to update").1 = location;
    }
}

#[cfg(test)]
mod test {
    use crate::game::locations::{LocationOrbit, LocationSpace};
    use crate::game::objects::ObjId;
    use crate::game::utils::{Position, Speed, TotalTime};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::{RunSystemOnce, SystemState};
    use commons::math::{deg_to_rads, P2};

    #[test]
    fn test_orbits_system_should_resolve_positions() {
        let mut world = World::new();
        world.insert_resource(TotalTime(0.0));
        let (star_id, planet1_id, planet2_id, planet2moon1_id, station_id) =
            create_system_1(&mut world);
        world.run_system_once(super::system_compute_orbits);

        let get_pos = |id: Entity| -> Position { world.get::<LocationSpace>(id).unwrap().pos };

        assert!(Position::ZERO.abs_diff_eq(get_pos(star_id), 0.01));
        assert!(Position::new(1.0, 0.0).abs_diff_eq(get_pos(planet1_id), 0.01));
        assert!(Position::new(0.0, 1.0).abs_diff_eq(get_pos(planet2_id), 0.01));
        assert!(Position::new(0.0, 1.5).abs_diff_eq(get_pos(planet2moon1_id), 0.01));
        assert!(Position::new(0.25, 1.5).abs_diff_eq(get_pos(station_id), 0.01));
    }

    #[test]
    fn test_orbits_system_should_move_over_time() {
        fn fetch_positions(world: &mut World, star_id: ObjId) -> Vec<P2> {
            let mut system_state: SystemState<Query<(Entity, &LocationSpace)>> =
                SystemState::new(world);
            let query = system_state.get(world);
            query
                .iter()
                .filter(|(id, _)| *id != star_id)
                .map(|(_, l)| l.pos)
                .collect()
        }

        let mut world = World::new();
        world.insert_resource(TotalTime(0.0));
        let (star_id, _planet1_id, _planet2_id, _planet2moon1_id, _station_id) =
            create_system_1(&mut world);
        world.run_system_once(super::system_compute_orbits);

        let positions0 = fetch_positions(&mut world, star_id);

        world.insert_resource(TotalTime(30.0));
        world.run_system_once(super::system_compute_orbits);

        let positions1 = fetch_positions(&mut world, star_id);
        for (a, b) in positions0.into_iter().zip(positions1.into_iter()) {
            assert_ne!(a, b);
        }
    }

    fn create_system_1(world: &mut World) -> (Entity, Entity, Entity, Entity, Entity) {
        let now = world.get_resource::<TotalTime>().unwrap().clone();
        let sector_id = world.spawn_empty().id();

        let star_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .id();

        let planet1_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .insert(LocationOrbit {
                parent_id: star_id,
                distance: 1.0,
                start_time: now,
                start_angle: 0.0,
                speed: Speed(500.0),
            })
            .id();

        let planet2_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .insert(LocationOrbit {
                parent_id: star_id,
                distance: 1.0,
                start_time: now,
                start_angle: deg_to_rads(90.0),
                speed: Speed(500.0),
            })
            .id();

        let planet2_moon1_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .insert(LocationOrbit {
                parent_id: planet2_id,
                distance: 0.5,
                start_angle: deg_to_rads(90.0),
                start_time: now,
                speed: Speed(500.0),
            })
            .id();

        let station_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .insert(LocationOrbit {
                parent_id: planet2_moon1_id,
                distance: 0.25,
                start_angle: deg_to_rads(0.0),
                start_time: now,
                speed: Speed(500.0),
            })
            .id();

        (
            star_id,
            planet1_id,
            planet2_id,
            planet2_moon1_id,
            station_id,
        )
    }
}
