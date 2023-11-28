use crate::game::extractables::Extractable;
use crate::game::loader::Loader;
use crate::game::locations::LocationOrbit;
use crate::game::orbit::Orbits;
use crate::game::sectors::Sector;
use crate::game::shipyard::Shipyard;
use crate::game::wares::Wares;
use crate::game::{conf, loader, sectors, shipyard, Game};
use crate::utils::TotalTime;
use commons::math::{P2, P2I, V2, V2I};
use commons::unwrap_or_continue;
use rand::prelude::*;
use space_galaxy::system_generator;
use specs::prelude::*;
use specs::World;
use std::collections::HashSet;

pub enum InitialCondition {
    Random { station_per_sector_density: f32 },
    Minimal,
    MinimalStations,
}

pub struct RandomMapCfg {
    pub size: (usize, usize),
    pub seed: u64,
    pub fleets: usize,
    pub universe_cfg: system_generator::UniverseCfg,
    pub initial_condition: InitialCondition,
    pub params: conf::Params,
}

pub fn load_random(game: &mut Game, cfg: &RandomMapCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(cfg.seed);

    let world = &mut game.world;

    // create sectors
    generate_sectors(world, cfg.size, rng.gen());
    add_bodies_to_sectors(world, rng.gen(), &cfg.universe_cfg);
    match cfg.initial_condition {
        InitialCondition::Random {
            station_per_sector_density,
        } => add_stations_random(world, rng.gen(), &cfg.params, station_per_sector_density),
        InitialCondition::Minimal => add_mothership(world, rng.gen(), &cfg.params),
        InitialCondition::MinimalStations => add_stations_minimal(world, rng.gen(), &cfg.params),
    }

    // add ships
    {
        let mut shipyards = vec![];

        // find shipyards
        {
            let entities = world.entities();
            let shipyard_storage = world.read_storage::<Shipyard>();
            for (e, _) in (&entities, &shipyard_storage).join() {
                shipyards.push(e);
            }
        }

        // add mandatory ships
        let shipyard = *commons::prob::select(&mut rng, &shipyards).unwrap();
        Loader::add_ship_miner(world, shipyard, 0.75, format!("miner-{}", 0));
        let shipyard = *commons::prob::select(&mut rng, &shipyards).unwrap();
        Loader::add_ship_trader(world, shipyard, 1.0, format!("trader-{}", 0));

        for i in 0..cfg.fleets {
            let shipyard = *commons::prob::select(&mut rng, &shipyards).unwrap();
            let choose = rng.gen_range(0..=1);
            let code = i + 2;
            if choose == 0 {
                Loader::add_ship_miner(world, shipyard, 0.75, format!("miner-{}", code));
            } else {
                Loader::add_ship_trader(world, shipyard, 1.0, format!("trader-{}", code));
            }
        }
    }

    // update index
    game.reindex_sectors();
}

pub fn generate_sectors(world: &mut World, size: (usize, usize), seed: u64) {
    let mut sectors_by_index = vec![];

    let galaxy = space_galaxy::galaxy_generator::Galaxy::new(space_galaxy::galaxy_generator::Cfg {
        seed,
        size: (size.0 as i32, size.1 as i32),
    });

    // add sectors
    for s in &galaxy.sectors.list {
        let pos = P2I::new(s.coords.x, s.coords.y);
        let sector_id =
            Loader::add_sector(world, pos, format!("sector {} {}", s.coords.x, s.coords.y));
        sectors_by_index.push((sector_id, pos));
    }
    // add portals
    let mut cached: HashSet<(usize, usize)> = Default::default();

    for j in &galaxy.jumps {
        if !cached.insert((j.sector_a, j.sector_b)) {
            continue;
        }

        if !cached.insert((j.sector_b, j.sector_a)) {
            continue;
        }

        Loader::add_jump(
            world,
            sectors_by_index[j.sector_a].0,
            j.pos_a.into(),
            sectors_by_index[j.sector_b].0,
            j.pos_b.into(),
        );
    }

    sectors::update_sectors_index(world);
}

fn add_bodies_to_sectors(
    world: &mut World,
    seed: u64,
    universe_cfg: &system_generator::UniverseCfg,
) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let sectors_id = sectors::list(world);
    let wares = Wares::list_wares_by_code(world);

    for sector_id in sectors_id {
        let system = system_generator::new_system(&universe_cfg, rng.gen());

        // create bodies
        let mut new_bodies = vec![];
        for body in &system.bodies {
            let maybe_obj_id = match &body.desc {
                system_generator::BodyDesc::Star { .. } => {
                    let new_obj = Loader::new_star(sector_id);
                    Some(Loader::add_object(world, &new_obj))
                }
                system_generator::BodyDesc::AsteroidField { resources } => {
                    let maybe_ware_id = resources
                        .iter()
                        .flat_map(|body_resource| {
                            let ware_id = wares.get(body_resource.resource.as_str());
                            if ware_id.is_none() {
                                log::warn!(
                                    "asteroid resource {:?} is not a ware, ignoring",
                                    body_resource.resource
                                );
                            }
                            ware_id
                        })
                        .next();

                    if let Some(ware_id) = maybe_ware_id {
                        let new_obj = Loader::new_asteroid(sector_id).extractable(Extractable {
                            ware_id,
                            accessibility: 1.0,
                        });
                        Some(Loader::add_object(world, &new_obj))
                    } else {
                        log::warn!("fail to create asteroid field {:?}", body);
                        None
                    }
                }
                system_generator::BodyDesc::Planet(_) => {
                    let new_obj = Loader::new_planet(sector_id);
                    Some(Loader::add_object(world, &new_obj))
                }
            };
            new_bodies.push((maybe_obj_id, body));
        }

        // update orbits
        let total_time = *world.read_resource::<TotalTime>();
        let mut orbits = world.write_storage::<LocationOrbit>();

        for (obj_id, body) in &new_bodies {
            let obj_id = unwrap_or_continue!(obj_id);

            if body.index == body.parent {
                continue;
            }

            // search body with parent
            let found = new_bodies.iter().find(|(_, j)| j.index == body.parent);

            let parent_obj_id = match found {
                Some((Some(id), _)) => id,
                _ => {
                    log::warn!(
                        "at sector {:?}, fail to find parent body for {:?}",
                        sector_id,
                        body.parent
                    );
                    continue;
                }
            };

            let speed = Loader::compute_orbit_speed(body.distance);
            let orbit = LocationOrbit {
                parent_id: *parent_obj_id,
                distance: body.distance,
                start_time: total_time,
                start_angle: body.angle,
                speed: speed,
            };
            log::trace!(
                "{:?} on orbit radius {:?} setted speed {:?}",
                obj_id,
                body.distance,
                speed.0
            );
            orbits.insert(*obj_id, orbit).unwrap();
        }
    }

    Orbits::update_orbits(world);
}

fn add_stations_random(
    world: &mut World,
    seed: u64,
    params: &conf::Params,
    station_per_sector_density: f32,
) {
    let rng: &mut StdRng = &mut SeedableRng::seed_from_u64(seed);

    // add minimal requirements
    add_stations_minimal(world, rng.next_u64(), params);

    // compute number of stations to add
    let sectors_list = sectors::list(world);
    let total_stations: f32 = station_per_sector_density * sectors_list.len() as f32;
    let total_solar = (total_stations / 3.0) as i32;
    let total_factory = (total_stations / 3.0) as i32;
    let total_shipyard = (total_stations / 10.0) as i32;

    let shipyard_new_obj =
        Loader::new_by_prefab_code(world, params.prefab_station_shipyard.as_str())
            .expect("fail to new shipyard");
    let factory_new_obj = Loader::new_by_prefab_code(world, params.prefab_station_factory.as_str())
        .expect("fail to new factory");
    let solar_new_obj = Loader::new_by_prefab_code(world, params.prefab_station_solar.as_str())
        .expect("fail to new solar");

    for (mut new_obj, count) in [
        (solar_new_obj, total_solar),
        (factory_new_obj, total_factory),
        (shipyard_new_obj, total_shipyard),
    ] {
        for _ in 0..count {
            // choose a sector
            let sector_id = commons::prob::select(rng, &sectors_list)
                .copied()
                .expect("empty list of sectors");

            // create obj in a random orbit
            new_obj = new_obj.at_position(sector_id, P2::ZERO);

            new_obj.shipyard.as_mut().map(|shipyard| {
                shipyard.set_production_order(shipyard::ProductionOrder::Random);
            });

            let obj_id = Loader::add_object(world, &new_obj);

            _ = loader::set_orbit_random_body(world, obj_id, rng.next_u64());
        }
    }

    Orbits::update_orbits(world);
}

fn add_mothership(world: &mut World, seed: u64, params: &conf::Params) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    // get first sector
    let sector_id = sectors::get_sector_by_coords(
        &world.entities(),
        &world.read_storage::<Sector>(),
        V2I::new(0, 0),
    )
    .expect("no sector found");

    // add shipyard
    let new_obj = Loader::new_by_prefab_code(world, params.prefab_mothership.as_str())
        .expect("fail to create mothership")
        .at_position(sector_id, V2::ZERO);
    let obj_id = Loader::add_object(world, &new_obj);
    _ = loader::set_orbit_random_body(world, obj_id, rng.next_u64());
}

fn add_stations_minimal(world: &mut World, seed: u64, params: &conf::Params) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    // get first sector
    let sector_id = sectors::get_sector_by_coords(
        &world.entities(),
        &world.read_storage::<Sector>(),
        V2I::new(0, 0),
    )
    .expect("no sector found");

    // add shipyard
    let new_obj = Loader::new_by_prefab_code(world, params.prefab_station_shipyard.as_str())
        .expect("fail to new shipyard")
        .at_position(sector_id, V2::ZERO);
    let obj_id = Loader::add_object(world, &new_obj);
    _ = loader::set_orbit_random_body(world, obj_id, rng.next_u64());

    // factory
    let new_obj = Loader::new_by_prefab_code(world, params.prefab_station_factory.as_str())
        .expect("fail to new factory")
        .at_position(sector_id, V2::ZERO);
    let obj_id = Loader::add_object(world, &new_obj);
    _ = loader::set_orbit_random_body(world, obj_id, rng.next_u64());

    // solar
    let new_obj = Loader::new_by_prefab_code(world, params.prefab_station_solar.as_str())
        .expect("fail to new solar")
        .at_position(sector_id, V2::ZERO);
    let obj_id = Loader::add_object(world, &new_obj);
    _ = loader::set_orbit_random_body(world, obj_id, rng.next_u64());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_random_scenery() {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .try_init();

        let path = "../data/game.conf";
        let file = std::fs::read_to_string(path).expect("fail to read config file");
        let cfg = conf::load_str(&file).expect("fail to read config file");

        let mut game = Game::new();
        loader::load_prefabs(&mut game.world, &cfg.prefabs);

        let rcfg = RandomMapCfg {
            size: (3, 3),
            seed: 0,
            fleets: 3,
            universe_cfg: cfg.system_generator.clone().unwrap(),
            initial_condition: InitialCondition::Random {
                station_per_sector_density: 1.0,
            },
            params: cfg.params,
        };

        load_random(&mut game, &rcfg);
    }
}
