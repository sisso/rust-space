use log::debug;
use rand::prelude::*;
use space_galaxy::system_generator::BodyDesc;
use space_galaxy::{galaxy_generator, system_generator, terrain_generator};
use std::cmp::max;
use std::path::{Path, PathBuf};

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let seed = 0u64;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let galaxy_cfg = galaxy_generator::Cfg {
        seed: rng.gen(),
        size: 2,
    };
    let system_cfg = system_generator::new_config(PathBuf::from("space-galaxy/data").as_path());

    let galaxy = galaxy_generator::Galaxy::new(galaxy_cfg);
    for sector in galaxy.sectors.list.iter() {
        let system = system_generator::new_system(&system_cfg, rng.gen());
        debug!("{:?}) {:?}", sector.id, system);

        for (i, body) in system.bodies.iter().enumerate() {
            debug!("{:?}", body);

            let body_desc = match &body.desc {
                BodyDesc::Planet(planet) => planet,
                _ => continue,
            };

            let min_size = 32.0;
            let max_size = 512.0;
            let max_body_size = 10.0;

            let size = commons::math::lerp(min_size, max_size, body.size / max_body_size);
            debug!("size: {}, dimension: {}", body.size, size);

            let width = size as i32;
            let height = (size * 0.75) as i32;

            let resources = body_desc.resources.iter().map(|i| i.amount).collect();

            let terrain = terrain_generator::generate_terrain(&terrain_generator::Cfg {
                width: width,
                height: height,
                seed: rng.gen(),
                heighmap_scale: 0.004,
                biomemap_scale: 0.007,
                shape: terrain_generator::Shape::Plain,
                resources,
            });

            let image = terrain_generator::generate_image(
                terrain.width,
                terrain.height,
                &terrain.height_map,
                &terrain.biomes_map,
            );
            image
                .save(&format!("/tmp/res/{}-{}-terrain.png", sector.id, i))
                .unwrap();

            for (j, r) in terrain.resources_map.iter().enumerate() {
                terrain_generator::save_gradient_as_image(
                    terrain.width,
                    terrain.height,
                    &terrain.resources_map[0],
                    &format!("/tmp/res/{}-{}-resource-{}.png", sector.id, i, j),
                );
            }
        }
    }

    // println!("{:?}", galaxy);
}
