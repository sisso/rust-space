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
use specs::prelude::*;
use std::collections::HashSet;

struct SceneryCfg {
    ware_ore_id: ObjId,
    ware_components_id: ObjId,
    ware_energy: ObjId,
    receipt_process_ores: Receipt,
    receipt_produce_energy: Receipt,
}

pub struct RandomMapCfg {
    pub size: usize,
    pub seed: u64,
    pub ships: usize,
    pub universe_cfg: system_generator::UniverseCfg,
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
            input: vec![
                WareAmount::new(ware_ore_id, 20),
                WareAmount::new(ware_energy, 10),
            ],
            output: vec![WareAmount::new(ware_components_id, 10)],
            time: DeltaTime(1.0),
        };
        let receipt_produce_energy = Receipt {
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
    add_stations(world, rng.gen(), &scenery_cfg);

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

        for i in 0..cfg.ships {
            let shipyard = commons::prob::select(&mut rng, &shipyards).unwrap();

            let choose = rng.gen_range(0..=1);
            if i == 0 || (i != 1 && choose == 0) {
                Loader::add_ship_miner(world, shipyard.to_owned(), 1.0, format!("miner-{}", i));
            } else {
                Loader::add_ship_trader(world, shipyard.to_owned(), 1.0, format!("trader-{}", i));
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
        let sector_id = Loader::add_sector(world, pos, format!("sector {}", s.id));
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

    let mut sectors_id = vec![];
    {
        let sectors = &world.read_storage::<Sector>();
        let entities = &world.entities();
        for (e, _) in (entities, sectors).join() {
            sectors_id.push(e);
        }
    }

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

fn add_stations(world: &mut World, seed: u64, scenery: &SceneryCfg) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let sector_kind_empty = 0;
    let sector_kind_asteroid = 1;
    let sector_kind_power = 2;
    let sector_kind_factory = 3;
    let sector_kind_prob = vec![
        commons::prob::Weighted {
            prob: 1.0,
            value: sector_kind_empty,
        },
        commons::prob::Weighted {
            prob: 1.0,
            value: sector_kind_asteroid,
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

    fn sector_pos<R: rand::Rng>(rng: &mut R) -> V2 {
        V2::new(
            (rng.gen_range(0..10) - 5) as f32,
            (rng.gen_range(0..10) - 5) as f32,
        )
    }

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
                Some(i) if *i == sector_kind_asteroid => {
                    required_kinds[0] = true;
                    let obj_id = Loader::add_asteroid(
                        world,
                        sector_id,
                        sector_pos(&mut rng),
                        scenery.ware_ore_id,
                    );
                    set_orbit_random_body(world, obj_id, rng.next_u64());
                }
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

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    pub fn test_random_scenery() {
        env_logger::builder()
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
                ships: 3,
                universe_cfg,
            },
        );
    }
}
