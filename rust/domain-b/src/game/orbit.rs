use crate::game::locations::{LocationOrbit, LocationSpace};
use crate::game::objects::ObjId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::{Speed, TotalTime};
use bevy_ecs::prelude::*;
use commons::math;
use commons::math::{Distance, Rad, P2};
use std::collections::HashMap;
use std::ops::Deref;

pub struct Orbits;

impl RequireInitializer for Orbits {
    fn init(context: &mut GameInitContext) {
        context.dispatcher.add(OrbitalPosSystem, "orbital_pos", &[]);
    }
}

impl Orbits {
    // id_map is mapping a giving astro index to a world entity
    pub fn update_orbits(world: &mut World) {
        let mut system = OrbitalPosSystem;
        system.run_now(world);
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
fn compute_orbital_pos<'a, D1, D2>(
    cache: &'a mut HashMap<ObjId, LocationSpace>,
    locations: &Storage<'a, LocationSpace, D1>,
    orbits: &Storage<'a, LocationOrbit, D2>,
    time: TotalTime,
    id: ObjId,
) -> Result<LocationSpace, &'static str>
where
    D1: Deref<Target = MaskedStorage<LocationSpace>>,
    D2: Deref<Target = MaskedStorage<LocationOrbit>>,
{
    if let Some(pos) = cache.get(&id) {
        return Ok(*pos);
    }

    let at_space = locations.get(id).ok_or("obj without position")?;
    let orbit = if let Some(orbit) = orbits.get(id) {
        orbit
    } else {
        cache.insert(id, *at_space);
        return Ok(*at_space);
    };

    let parent_pos = match compute_orbital_pos(cache, locations, orbits, time, orbit.parent_id) {
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

pub struct OrbitalPosSystem;

impl<'a> System<'a> for OrbitalPosSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, TotalTime>,
        WriteStorage<'a, LocationSpace>,
        ReadStorage<'a, LocationOrbit>,
    );

    fn run(&mut self, (entities, time, mut locations_space, orbits): Self::SystemData) {
        log::trace!("running");

        let mut cache: HashMap<ObjId, LocationSpace> = Default::default();
        let mut updates = vec![];

        let current_time = *time;

        for (obj_id, _) in (&entities, &orbits).join() {
            let pos = match compute_orbital_pos(
                &mut cache,
                &locations_space,
                &orbits,
                current_time,
                obj_id,
            ) {
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

        let loc_space = &mut locations_space;
        for (obj_id, location) in updates {
            log::trace!("{:?} updating orbit location to {:?}", obj_id, location);
            loc_space.insert(obj_id, location).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::locations::{LocationOrbit, LocationSpace};
    use crate::game::objects::ObjId;
    use crate::game::orbit::OrbitalPosSystem;
    use crate::test::TestSystemRunner;
    use crate::utils::{Position, Speed, TotalTime};
    use bevy_ecs::prelude::*;
    use commons::math::{deg_to_rads, P2};
    use specs::World;
    use specs::{Builder, Entity, WorldExt};

    #[test]
    fn test_orbits_system_should_resolve_positions() {
        let (world, result) = crate::test::test_system(OrbitalPosSystem, move |world| {
            world.insert(TotalTime(0.0));
            create_system_1(world)
        });

        let (star_id, planet1_id, planet2_id, planet2moon1_id, station_id) = result;
        let locations = world.read_storage::<LocationSpace>();

        let get_pos = |id: Entity| -> Position { locations.get(id).unwrap().pos };

        assert!(Position::ZERO.abs_diff_eq(get_pos(star_id), 0.01));
        assert!(Position::new(1.0, 0.0).abs_diff_eq(get_pos(planet1_id), 0.01));
        assert!(Position::new(0.0, 1.0).abs_diff_eq(get_pos(planet2_id), 0.01));
        assert!(Position::new(0.0, 1.5).abs_diff_eq(get_pos(planet2moon1_id), 0.01));
        assert!(Position::new(0.25, 1.5).abs_diff_eq(get_pos(station_id), 0.01));
    }

    #[test]
    fn test_orbits_system_should_move_over_time() {
        fn fetch_positions(world: &World, star_id: ObjId) -> Vec<P2> {
            (&world.entities(), &world.read_component::<LocationSpace>())
                .join()
                .filter(|(id, _)| *id != star_id)
                .map(|(_, l)| l.pos)
                .collect()
        }

        let mut runner = TestSystemRunner::new(OrbitalPosSystem);
        runner.world.insert(TotalTime(0.0));
        let (star_id, _, _, _, _) = create_system_1(&mut runner.world);
        runner.tick();

        let positions0 = fetch_positions(&runner.world, star_id);

        runner.world.insert(TotalTime(30.0));
        runner.tick();

        let positions1 = fetch_positions(&runner.world, star_id);

        for (a, b) in positions0.into_iter().zip(positions1.into_iter()) {
            assert_ne!(a, b);
        }
    }

    fn create_system_1(world: &mut World) -> (Entity, Entity, Entity, Entity, Entity) {
        let now = *world.read_resource::<TotalTime>();
        let sector_id = world.create_entity().build();

        let star_id = world
            .create_entity()
            .with(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .build();

        let planet1_id = world
            .create_entity()
            .with(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .with(LocationOrbit {
                parent_id: star_id,
                distance: 1.0,
                start_time: now,
                start_angle: 0.0,
                speed: Speed(500.0),
            })
            .build();

        let planet2_id = world
            .create_entity()
            .with(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .with(LocationOrbit {
                parent_id: star_id,
                distance: 1.0,
                start_time: now,
                start_angle: deg_to_rads(90.0),
                speed: Speed(500.0),
            })
            .build();

        let planet2_moon1_id = world
            .create_entity()
            .with(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .with(LocationOrbit {
                parent_id: planet2_id,
                distance: 0.5,
                start_angle: deg_to_rads(90.0),
                start_time: now,
                speed: Speed(500.0),
            })
            .build();

        let station_id = world
            .create_entity()
            .with(LocationSpace {
                pos: Position::ZERO,
                sector_id,
            })
            .with(LocationOrbit {
                parent_id: planet2_moon1_id,
                distance: 0.25,
                start_angle: deg_to_rads(0.0),
                start_time: now,
                speed: Speed(500.0),
            })
            .build();

        (
            star_id,
            planet1_id,
            planet2_id,
            planet2_moon1_id,
            station_id,
        )
    }
}
