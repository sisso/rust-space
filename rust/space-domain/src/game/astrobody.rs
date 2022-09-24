use crate::game::locations::Location;
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
    // pub parent_entity: Option<Entity>,
    pub system_index: usize,
    pub parent_index: usize,
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
}

pub struct OrbitalPosSystem;

fn find_orbital_pos(bodies: &Vec<&OrbitalPos>, system_index: usize) -> Position {
    for b in bodies {
        if b.system_index == system_index {
            let local = b.compute_local_pos();

            log::trace!("local pos of {} is {:?}", system_index, local);

            let pos = if b.system_index == b.parent_index {
                local
            } else {
                let parent_pos = find_orbital_pos(bodies, b.parent_index);
                parent_pos.add(&local)
            };
            log::trace!("pos of {} is {:?}", system_index, pos);
            return pos;
        }
    }

    panic!("fail to find system index");
}

impl<'a> System<'a> for OrbitalPosSystem {
    type SystemData = (
        Read<'a, TotalTime>,
        WriteStorage<'a, Location>,
        ReadStorage<'a, OrbitalPos>,
    );

    fn run(&mut self, (time, mut locations, orbits): Self::SystemData) {
        log::trace!("running");

        let mut bodies_by_systems = HashMap::new();

        // organize orbital bodies by system
        for (orbit, location) in (&orbits, &mut locations).join() {
            if let Some(sector_id) = location.as_space().map(|i| i.sector_id) {
                bodies_by_systems
                    .entry(sector_id)
                    .or_insert_with(|| vec![])
                    .push((orbit, location));
            }
        }

        // resolve positions
        for (system_id, bodies) in bodies_by_systems {
            log::trace!(
                "checking sector {}, bodies {}",
                system_id.id(),
                bodies.len()
            );
            let bodies_only = bodies.iter().map(|(o, _)| *o).collect::<Vec<_>>();
            let mut positions = vec![];
            for i in 0..bodies_only.len() {
                let pos = find_orbital_pos(&bodies_only, i);
                positions.push(pos);
            }

            for ((orbit, location), pos) in bodies.into_iter().zip(positions.into_iter()) {
                log::trace!(
                    "updating {} on {:?} to {:?}",
                    orbit.system_index,
                    location,
                    pos
                );
                (*location).set_pos(pos).unwrap();
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
                .with(OrbitalPos {
                    system_index: 0,
                    parent_index: 0,
                    distance: 0.0,
                    initial_angle: 0.0,
                })
                .build();

            let planet1_id = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::zero(),
                    sector_id,
                })
                .with(OrbitalPos {
                    system_index: 1,
                    parent_index: 0,
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
                    system_index: 2,
                    parent_index: 0,
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
                    system_index: 3,
                    parent_index: 2,
                    distance: 0.5,
                    initial_angle: deg_to_rads(90.0),
                })
                .build();

            (star_id, planet1_id, planet2_id, planet2_moon1_id)
        });

        let (star_id, planet1_id, planet2_id, planet2moon1_id) = result;

        let locations = world.read_storage::<Location>();

        let get_pos =
            |id: Entity| -> Position { locations.get(id).unwrap().as_space().unwrap().pos };

        assert_eq!(Position::zero(), get_pos(star_id));
        assert_eq!(Position::new(1.0, 0.0), get_pos(planet1_id));
        assert_eq!(Position::new(0.0, 1.0), get_pos(planet2_id));
        assert_eq!(Position::new(0.0, 1.5), get_pos(planet2moon1_id));
    }
}
