use crate::game::locations::Location;
use crate::game::objects::ObjId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::TotalTime;
use commons::math;
use commons::math::{Rad, P2};
use specs::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum AstroBodyKind {
    Star,
    Planet,
}

#[derive(Clone, Debug, Component)]
pub struct AstroBody {
    pub kind: AstroBodyKind,
}

#[derive(Clone, Debug, Component)]
pub struct OrbitalPos {
    pub parent: ObjId,
    pub distance: f32,
    pub initial_angle: Rad,
}

impl OrbitalPos {
    pub fn compute_local_pos(&self, time: TotalTime) -> P2 {
        let speed = 500.0f64;
        let angle = self.initial_angle as f64 + time.0 * (math::TWO_PI as f64 / speed);
        math::rotate_vector_by_angle(math::P2::new(self.distance, 0.0), angle as f32).into()
    }
}

pub struct AstroBodies;

impl RequireInitializer for AstroBodies {
    fn init(context: &mut GameInitContext) {
        context.world.register::<AstroBody>();
        context.world.register::<OrbitalPos>();

        context.dispatcher.add(OrbitalPosSystem, "orbital_pos", &[]);
    }
}

impl AstroBodies {
    // id_map is mapping a giving astro index to a world entity
    pub fn update_orbits(world: &mut World) {
        let mut system = OrbitalPosSystem;
        system.run_now(world);
    }

    pub fn set_orbits_from_storage(
        orbits: &mut WriteStorage<OrbitalPos>,
        obj_id: ObjId,
        parent_id: ObjId,
        radius: f32,
        angle: f32,
    ) {
        orbits
            .insert(
                obj_id,
                OrbitalPos {
                    parent: parent_id,
                    distance: radius,
                    initial_angle: angle,
                },
            )
            .unwrap();
    }

    pub fn set_orbit(world: &mut World, obj_id: ObjId, parent_id: ObjId, radius: f32, angle: f32) {
        let mut orbits = world.write_storage::<OrbitalPos>();
        Self::set_orbits_from_storage(&mut orbits, obj_id, parent_id, radius, angle);
    }
}

pub struct OrbitalPosSystem;

fn find_orbital_pos(bodies: &Vec<(ObjId, &OrbitalPos)>, time: TotalTime, id: ObjId) -> Option<P2> {
    for (i, b) in bodies {
        if *i != id {
            continue;
        }

        let local = b.compute_local_pos(time);
        log::trace!("find_orbital_pos local pos of {:?} is {:?}", id, local);

        let pos = match find_orbital_pos(bodies, time, b.parent) {
            Some(parent_pos) => parent_pos + local,
            None => local,
        };
        log::trace!("find_orbital_pos pos of {:?} is {:?}", id, pos);
        return Some(pos);
    }

    log::trace!("find_orbital_pos found no orbital pos for {:?}", id);
    None
}

impl<'a> System<'a> for OrbitalPosSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, TotalTime>,
        WriteStorage<'a, Location>,
        ReadStorage<'a, OrbitalPos>,
    );

    fn run(&mut self, (entities, time, mut locations, orbits): Self::SystemData) {
        log::trace!("running");

        let mut bodies_by_systems = HashMap::new();

        // organize orbital bodies by system
        for (id, orbit, location) in (&entities, &orbits, &mut locations).join() {
            if let Some(sector_id) = location.as_space().map(|i| i.sector_id) {
                bodies_by_systems
                    .entry(sector_id)
                    .or_insert_with(|| vec![])
                    .push((id, orbit, location));
            }
        }

        // resolve positions
        for (sector_id, bodies) in bodies_by_systems {
            log::trace!(
                "checking sector {}, bodies {}",
                sector_id.id(),
                bodies.len()
            );

            let bodies_only = bodies
                .iter()
                .map(|(id, o, _)| (*id, *o))
                .collect::<Vec<_>>();

            for (id, _, loc) in bodies {
                match find_orbital_pos(&bodies_only, *time, id) {
                    Some(pos) => {
                        log::trace!("updating {:?} on {:?} to {:?}", id, loc, pos);
                        (*loc).set_pos(pos).unwrap();
                    }

                    None => {
                        log::trace!("fail to updating {:?} on {:?} to None", id, loc,);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::astrobody::{OrbitalPos, OrbitalPosSystem};
    use crate::game::locations::Location;
    use crate::utils::{Position, TotalTime};
    use approx::assert_abs_diff_eq;
    use commons::math::deg_to_rads;
    use shred::World;
    use specs::{Builder, Entity, WorldExt};

    #[test]
    fn test_orbits_system_should_resolve_positions() {
        crate::test::init_log();

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
        crate::test::init_log();

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
