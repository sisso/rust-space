use crate::game::locations::Location;
use crate::game::objects::ObjId;
use crate::game::{GameInitContext, RequireInitializer};
use crate::utils::{Position, TotalTime};
use commons::math;
use commons::math::Rad;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
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
    pub fn compute_local_pos(&self) -> Position {
        math::rotate_vector_by_angle(math::P2::new(self.distance, 0.0), self.initial_angle).into()
    }
}

pub struct AstroBodies;

impl RequireInitializer for AstroBodies {
    fn init(context: &mut GameInitContext) {
        context.world.register::<AstroBody>();
        context.world.register::<OrbitalPos>();
    }
}

impl AstroBodies {
    // id_map is mapping a giving astro index to a world entity
    pub fn update_orbits(world: &mut World) {
        let mut system = OrbitalPosSystem;
        system.run_now(world);
    }

    pub fn set_orbit_2(
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
        AstroBodies::set_orbit_2(&mut orbits, obj_id, parent_id, radius, angle);
    }
}

pub struct OrbitalPosSystem;

fn find_orbital_pos(bodies: &Vec<(ObjId, &OrbitalPos)>, id: ObjId) -> Option<Position> {
    for (i, b) in bodies {
        if *i != id {
            continue;
        }

        let local = b.compute_local_pos();
        log::trace!("find_orbital_pos local pos of {:?} is {:?}", id, local);

        let pos = match find_orbital_pos(bodies, b.parent) {
            Some(parent_pos) => parent_pos.add(&local),
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
        Read<'a, TotalTime>,
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
                match find_orbital_pos(&bodies_only, id) {
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
    use crate::utils::Position;
    use approx::{assert_relative_eq, relative_eq};
    use commons::math;
    use commons::math::deg_to_rads;
    use specs::{Builder, Entity, WorldExt};

    #[test]
    fn test_orbits_system_should_resolve_positions() {
        crate::test::init_log();

        let (world, result) = crate::test::test_system(OrbitalPosSystem, move |world| {
            let sector_id = world.create_entity().build();

            let star_id = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::zero(),
                    sector_id,
                })
                .build();

            let planet1_id = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::zero(),
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
                    pos: Position::zero(),
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
                    pos: Position::zero(),
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
                    pos: Position::zero(),
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
        });

        let (star_id, planet1_id, planet2_id, planet2moon1_id, station_id) = result;
        log::trace!(
            "star {:?}, planet 1 {:?}, planet 2 {:?}, moon {:?} station {:?}",
            star_id,
            planet1_id,
            planet2_id,
            planet2moon1_id,
            station_id
        );

        let locations = world.read_storage::<Location>();

        let get_pos =
            |id: Entity| -> Position { locations.get(id).unwrap().as_space().unwrap().pos };

        assert_eq!(Position::zero(), get_pos(star_id));
        assert_eq!(Position::new(1.0, 0.0), get_pos(planet1_id));
        assert_eq!(Position::new(0.0, 1.0), get_pos(planet2_id));
        assert_eq!(Position::new(0.0, 1.5), get_pos(planet2moon1_id));
        assert_eq!(Position::new(0.25, 1.5), get_pos(station_id));
    }
}
