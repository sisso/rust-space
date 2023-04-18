use crate::game::astrobody::{AstroBodies, OrbitalPos};
use crate::game::extractables::Extractable;
use crate::game::factory::Receipt;
use crate::game::loader::*;
use crate::game::objects::ObjId;
use crate::game::sectors::Sector;
use crate::game::shipyard::Shipyard;
use crate::game::wares::WareAmount;
use crate::game::{sectors, Game};
use crate::utils::DeltaTime;
use commons::math::V2;
use commons::unwrap_or_continue;
use rand::prelude::*;
use shred::World;
use space_galaxy::system_generator;
use space_galaxy::system_generator::BodyDesc::AsteroidField;
use specs::prelude::*;
use std::collections::HashSet;

struct SceneryCfg {
    ware_ore_id: ObjId,
    ware_components_id: ObjId,
    ware_energy: ObjId,
    receipt_process_ores: Receipt,
    receipt_produce_energy: Receipt,
}

pub enum InitialCondition {
    Random,
    Minimal,
}

pub struct RandomMapCfg {
    pub size: usize,
    pub seed: u64,
    pub fleets: usize,
    pub universe_cfg: system_generator::UniverseCfg,
    pub initial_condition: InitialCondition,
}

pub fn load_random(game: &mut Game, cfg: &RandomMapCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(cfg.seed);

    let world = &mut game.world;

    // add configurations
    let scenery_cfg = {
        // wares and receipts
        let ware_ore_id = Loader::add_ware(world, "ore".to_string());
        let ware_components_id = Loader::add_ware(world, "components".to_string());
        let ware_energy = Loader::add_ware(world, "energy".to_string());

        let receipt_process_ores = Receipt {
            label: "ore processing".to_string(),
            input: vec![
                WareAmount::new(ware_ore_id, 20),
                WareAmount::new(ware_energy, 10),
            ],
            output: vec![WareAmount::new(ware_components_id, 10)],
            time: DeltaTime(1.0),
        };
        let receipt_produce_energy = Receipt {
            label: "solar power".to_string(),
            input: vec![],
            output: vec![WareAmount::new(ware_energy, 10)],
            time: DeltaTime(5.0),
        };

        SceneryCfg {
            ware_ore_id,
            ware_components_id,
            ware_energy,
            receipt_process_ores,
            receipt_produce_energy,
        }
    };

    // create sectors
    generate_sectors(world, cfg.size, rng.gen());
    add_bodies_to_sectors(world, rng.gen(), &cfg.universe_cfg, &scenery_cfg);
    add_asteroid_fields_to_sectors(world, rng.gen(), &scenery_cfg);
    match cfg.initial_condition {
        InitialCondition::Random { .. } => add_stations_random(world, rng.gen(), &scenery_cfg),
        InitialCondition::Minimal => add_stations_minimal(world, rng.gen(), &scenery_cfg),
    }

    // add ships
    {
        let mut shipyards = vec![];

        {
            let entities = world.entities();
            let shipyard_storage = world.read_storage::<Shipyard>();
            for (e, _) in (&entities, &shipyard_storage).join() {
                shipyards.push(e);
            }
        }

        // add mandatory ships
        let shipyard = *commons::prob::select(&mut rng, &shipyards).unwrap();
        Loader::add_ship_miner(world, shipyard, 1.0, format!("miner-{}", 0));
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

pub fn generate_sectors(world: &mut World, size: usize, seed: u64) {
    let mut sectors_by_index = vec![];

    let galaxy = space_galaxy::galaxy_generator::Galaxy::new(space_galaxy::galaxy_generator::Cfg {
        seed,
        size: size as i32,
    });

    // add sectors
    for s in &galaxy.sectors.list {
        let pos = V2::new(s.coords.x as f32, s.coords.y as f32);
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
    scenery_cfg: &SceneryCfg,
) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let sectors_id = list_sectors(&world);

    for sector_id in sectors_id {
        let system = system_generator::new_system(&universe_cfg, rng.gen());

        // create bodies
        let mut new_bodies = vec![];
        for body in &system.bodies {
            let maybe_obj_id = match body.desc {
                system_generator::BodyDesc::Star { .. } => {
                    let new_obj = Loader::new_star(sector_id);
                    Some(Loader::add_object(world, &new_obj))
                }
                system_generator::BodyDesc::AsteroidField { .. } => {
                    let new_obj = Loader::new_asteroid(sector_id).extractable(Extractable {
                        ware_id: scenery_cfg.ware_ore_id,
                    });
                    Some(Loader::add_object(world, &new_obj))
                }
                system_generator::BodyDesc::Planet(_) => {
                    let new_obj = Loader::new_planet(sector_id);
                    Some(Loader::add_object(world, &new_obj))
                }
            };
            new_bodies.push((maybe_obj_id, body));
        }

        // update orbits
        let mut orbits = world.write_storage::<OrbitalPos>();

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

            AstroBodies::set_orbit_2(
                &mut orbits,
                *obj_id,
                *parent_obj_id,
                body.distance,
                body.angle,
            );
        }
    }

    AstroBodies::update_orbits(world);
}

fn sector_pos<R: rand::Rng>(rng: &mut R) -> V2 {
    V2::new(
        (rng.gen_range(0..10) - 5) as f32,
        (rng.gen_range(0..10) - 5) as f32,
    )
}

fn list_sectors(world: &World) -> Vec<Entity> {
    (&world.entities(), &world.read_storage::<Sector>())
        .join()
        .map(|(e, _)| e)
        .collect()
}

fn add_stations_random(world: &mut World, seed: u64, scenery: &SceneryCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let sector_kind_empty = 0;
    let sector_kind_power = 1;
    let sector_kind_factory = 2;
    let sector_kind_prob = vec![
        commons::prob::Weighted {
            prob: 1.0,
            value: sector_kind_empty,
        },
        commons::prob::Weighted {
            prob: 1.0,
            value: sector_kind_factory,
        },
        commons::prob::Weighted {
            prob: 1.0,
            value: sector_kind_power,
        },
    ];

    let mut sectors_id = vec![];
    {
        let entities = world.entities();
        let sectors_repo = world.read_storage::<Sector>();
        for (sector_id, _) in (&entities, &sectors_repo).join() {
            sectors_id.push(sector_id);
        }
    }

    // adding shipyard
    {
        let sector_id = commons::prob::select(&mut rng, &sectors_id).expect("empty sectors_id");
        let obj_id = Loader::add_shipyard(
            world,
            *sector_id,
            sector_pos(&mut rng),
            scenery.ware_components_id,
        );
        set_orbit_random_body(world, obj_id, rng.next_u64());
    }

    let mut required_kinds = [false, false, false];
    while required_kinds.iter().any(|i| !*i) {
        for &sector_id in &sectors_id {
            let kind = commons::prob::select_weighted(&mut rng, &sector_kind_prob);

            log::info!("creating {:?} on sector {:?}", kind, sector_id);

            match kind {
                Some(i) if *i == sector_kind_factory => {
                    required_kinds[1] = true;

                    let obj_id = Loader::add_factory(
                        world,
                        sector_id,
                        sector_pos(&mut rng),
                        scenery.receipt_process_ores.clone(),
                    );
                    set_orbit_random_body(world, obj_id, rng.next_u64());
                }
                Some(i) if *i == sector_kind_power => {
                    required_kinds[2] = true;

                    let obj_id = Loader::add_factory(
                        world,
                        sector_id,
                        sector_pos(&mut rng),
                        scenery.receipt_produce_energy.clone(),
                    );
                    set_orbit_random_body(world, obj_id, rng.next_u64());
                }
                _ => {
                    log::warn!("unknown weight {:?}", kind);
                }
            }
        }
    }

    AstroBodies::update_orbits(world);
}
fn add_asteroid_fields_to_sectors(world: &mut World, seed: u64, scenery: &SceneryCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    // we only execute if world generation has no asteroid
    if !world.read_storage::<Extractable>().is_empty() {
        return;
    }

    let sectors = list_sectors(&world);
    let sector_id = *commons::prob::select_array(&mut rng, sectors.as_slice());
    let obj_id = Loader::add_object(
        world,
        &Loader::new_asteroid(sector_id)
            .extractable(Extractable {
                ware_id: scenery.ware_ore_id,
            })
            .with_label("ore asteroid".to_string()),
    );
    set_orbit_random_body(world, obj_id, rng.next_u64());
}

fn add_stations_minimal(world: &mut World, seed: u64, scenery: &SceneryCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let (sector_id, _) = (&world.entities(), &world.read_storage::<Sector>())
        .join()
        .next()
        .expect("no sector found");

    // add shipyard
    let obj_id = Loader::add_shipyard(world, sector_id, V2::ZERO, scenery.ware_components_id);
    set_orbit_random_body(world, obj_id, rng.next_u64());

    // add factory
    let obj_id = Loader::add_object(
        world,
        &Loader::new_factory(sector_id, V2::ZERO, scenery.receipt_process_ores.clone()),
    );
    set_orbit_random_body(world, obj_id, rng.next_u64());

    // add power generation
    let obj_id = Loader::add_object(
        world,
        &Loader::new_factory(sector_id, V2::ZERO, scenery.receipt_produce_energy.clone()),
    );
    set_orbit_random_body(world, obj_id, rng.next_u64());
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    pub fn test_random_scenery() {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .try_init();

        let mut game = Game::new();
        let universe_cfg = space_galaxy::system_generator::new_config_from_file(&PathBuf::from(
            "../data/system_generator.conf",
        ));
        load_random(
            &mut game,
            &RandomMapCfg {
                size: 3,
                seed: 0,
                fleets: 3,
                universe_cfg,
                initial_condition: InitialCondition::Random,
            },
        );
    }
}
