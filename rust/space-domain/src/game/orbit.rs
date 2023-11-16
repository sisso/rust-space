use crate::game::locations::{Location, LocationOrbit, LocationSpace, Orbit};
use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::{Speed, TotalTime, V2};
use commons::math::{Distance, Rad, P2};
use commons::{math, unwrap_or_continue};
use specs::prelude::*;
use specs::storage::MaskedStorage;
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

    pub fn set_orbits_from_storage(
        locations: &mut WriteStorage<Location>,
        obj_id: ObjId,
        parent_id: ObjId,
        radius: Distance,
        angle: Rad,
        speed: Speed,
        orbit_start_time: TotalTime,
    ) -> Result<(), ()> {
        let parent_position = locations.get(parent_id).ok_or(())?;
        let parent_space_pos = parent_position.as_space().ok_or(())?;

        locations
            .insert(
                obj_id,
                Location::Orbiting {
                    parent_id,
                    pos: Default::default(),
                    sector_id: parent_space_pos.sector_id,
                    orbit: Orbit {
                        radius,
                        starting: orbit_start_time,
                        start_angle: angle,
                        speed,
                    },
                },
            )
            .unwrap();

        Ok(())
    }

    pub fn set_orbit(
        world: &mut World,
        obj_id: ObjId,
        parent_id: ObjId,
        radius: Distance,
        angle: Rad,
        speed: Speed,
    ) -> Result<(), ()> {
        let total_time = *world.read_resource::<TotalTime>();
        let mut orbits = world.write_storage::<Location>();
        Self::set_orbits_from_storage(
            &mut orbits,
            obj_id,
            parent_id,
            radius,
            angle,
            speed,
            total_time,
        )
    }
}

pub fn compute_orbit_local_pos(
    radius: Distance,
    initial_angle: Rad,
    start_time: TotalTime,
    speed: Speed,
    current_time: TotalTime,
) -> P2 {
    let angle = initial_angle as f64
        + (current_time.as_f64() - start_time.as_f64()) * (math::TWO_PI as f64 / speed.0 as f64);
    math::rotate_vector_by_angle(P2::new(radius, 0.0), angle as f32).into()
}

/// try to location space for a giving orbit, will fail to resolve if the obj or any orbit parent
/// is docked
fn compute_orbital_pos<'a, D>(
    cache: &'a mut HashMap<ObjId, LocationSpace>,
    locations: &Storage<'a, Location, D>,
    time: TotalTime,
    id: ObjId,
) -> Result<LocationSpace, &'static str>
where
    D: Deref<Target = MaskedStorage<Location>>,
{
    if let Some(pos) = cache.get(&id) {
        return Ok(*pos);
    }

    let location = locations.get(id).ok_or("obj without location")?;
    match location {
        Location::Dock { .. } => Err("object is docked"),
        Location::Space { pos, sector_id } => {
            let pos = LocationSpace {
                pos: *pos,
                sector_id: *sector_id,
            };
            cache.insert(id, pos);
            Ok(pos)
        }
        Location::Orbiting {
            parent_id,
            pos,
            sector_id,
            orbit,
        } => {
            let parent_pos = match compute_orbital_pos(cache, locations, time, *parent_id) {
                Ok(pos) => pos,
                Err(err_msg) => {
                    log::warn!(
                        "{:?} fail to compute parent position for orbit {:?}: {}",
                        id,
                        parent_id,
                        err_msg
                    );
                    return Err("parent object is docked");
                }
            };

            let local_pos = compute_orbit_local_pos(
                orbit.radius,
                orbit.start_angle,
                orbit.starting,
                orbit.speed,
                time,
            );
            let globla_pos = parent_pos.pos + local_pos;
            let pos = LocationSpace {
                pos: globla_pos,
                sector_id: parent_pos.sector_id,
            };
            cache.insert(id, pos);
            Ok(pos)
        }
    }
}

pub struct OrbitalPosSystem;

impl<'a> System<'a> for OrbitalPosSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, TotalTime>,
        WriteStorage<'a, Location>,
    );

    fn run(&mut self, (entities, time, mut locations): Self::SystemData) {
        log::trace!("running");

        let mut cache: HashMap<ObjId, LocationSpace> = Default::default();
        let mut updates = vec![];

        let current_time = *time;

        for (obj_id, location) in (&entities, &locations).join() {
            let previous_location = unwrap_or_continue!(location.as_orbit());

            let pos = match compute_orbital_pos(&mut cache, &locations, current_time, obj_id) {
                Ok(pos) => pos,
                Err(err_msg) => {
                    log::warn!("{:?} fail to generate orbiting by {}", obj_id, err_msg);
                    continue;
                }
            };

            updates.push((
                obj_id,
                Location::Orbiting {
                    parent_id: previous_location.parent_id,
                    pos: pos.pos,
                    sector_id: pos.sector_id,
                    orbit: previous_location.orbit,
                },
            ));
        }

        let locations = &mut locations;
        for (obj_id, location) in updates {
            log::trace!("{:?} updating orbit location to {:?}", obj_id, location);
            locations.insert(obj_id, location).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::astrobody::{OrbitalPos, OrbitalPosSystem};
    use crate::game::locations::Location;
    use crate::utils::{Position, TotalTime};
    use commons::math::deg_to_rads;
    use specs::World;
    use specs::{Builder, Entity, WorldExt};

    #[test]
    fn test_orbits_system_should_resolve_positions() {
        crate::test::init_trace_log();

        let (world, result) = crate::test::test_system(OrbitalPosSystem, move |world| {
            world.insert(TotalTime(0.0));
            create_system_1(world)
        });

        let (star_id, planet1_id, planet2_id, planet2moon1_id, station_id) = result;
        let locations = world.read_storage::<Location>();

        let get_pos =
            |id: Entity| -> Position { locations.get(id).unwrap().as_space().unwrap().pos };

        assert!(Position::ZERO.abs_diff_eq(get_pos(star_id), 0.01));
        assert!(Position::new(1.0, 0.0).abs_diff_eq(get_pos(planet1_id), 0.01));
        assert!(Position::new(0.0, 1.0).abs_diff_eq(get_pos(planet2_id), 0.01));
        assert!(Position::new(0.0, 1.5).abs_diff_eq(get_pos(planet2moon1_id), 0.01));
        assert!(Position::new(0.25, 1.5).abs_diff_eq(get_pos(station_id), 0.01));
    }

    #[test]
    fn test_orbits_system_should_move_over_time() {
        crate::test::init_trace_log();

        let (world1, result1) = crate::test::test_system(OrbitalPosSystem, move |world| {
            world.insert(TotalTime(0.0));
            create_system_1(world)
        });

        let (world2, result2) = crate::test::test_system(OrbitalPosSystem, move |world| {
            world.insert(TotalTime(30.0));
            create_system_1(world)
        });

        let get_pos = |world: &World, id: Entity| -> Position {
            let locations = world.read_storage::<Location>();
            locations.get(id).unwrap().as_space().unwrap().pos
        };

        assert_ne!(get_pos(&world1, result1.4), get_pos(&world2, result2.4));
    }

    fn create_system_1(world: &mut World) -> (Entity, Entity, Entity, Entity, Entity) {
        let sector_id = world.create_entity().build();

        let star_id = world
            .create_entity()
            .with(Location::Space {
                pos: Position::ZERO,
                sector_id,
            })
            .build();

        let planet1_id = world
            .create_entity()
            .with(Location::Space {
                pos: Position::ZERO,
                sector_id,
            })
            .with(OrbitalPos {
                parent: star_id,
                distance: 1.0,
                initial_angle: 0.0,
            })
            .build();

        let planet2_id = world
            .create_entity()
            .with(Location::Space {
                pos: Position::ZERO,
                sector_id,
            })
            .with(OrbitalPos {
                parent: star_id,
                distance: 1.0,
                initial_angle: deg_to_rads(90.0),
            })
            .build();

        let planet2_moon1_id = world
            .create_entity()
            .with(Location::Space {
                pos: Position::ZERO,
                sector_id,
            })
            .with(OrbitalPos {
                parent: planet2_id,
                distance: 0.5,
                initial_angle: deg_to_rads(90.0),
            })
            .build();

        let station_id = world
            .create_entity()
            .with(Location::Space {
                pos: Position::ZERO,
                sector_id,
            })
            .with(OrbitalPos {
                parent: planet2_moon1_id,
                distance: 0.25,
                initial_angle: deg_to_rads(0.0),
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
