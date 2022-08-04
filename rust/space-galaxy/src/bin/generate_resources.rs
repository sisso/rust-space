use log::debug;
use rand::prelude::*;
use space_galaxy::terrain_generator::ResCfg;
use space_galaxy::{galaxy_generator, system_generator, terrain_generator};
use std::cmp::max;
use std::path::{Path, PathBuf};

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let width = 300;
    let height = 200;
    let params = vec![(1.0, 0.5), (0.5, 0.75), (0.25, 1.0), (1.0, 0.1)]
        .into_iter()
        .map(|(k, v)| ResCfg { amount: k, disp: v })
        .collect::<Vec<_>>();

    std::fs::create_dir_all("/tmp/res").unwrap();

    for (i, p) in params.iter().enumerate() {
        let seed = 0u64;
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

        let terrain = terrain_generator::generate_terrain(&terrain_generator::Cfg {
            width: width,
            height: height,
            seed: rng.gen(),
            heighmap_scale: 0.004,
            biomemap_scale: 0.007,
            shape: terrain_generator::Shape::Plain,
            resources: vec![p.clone()],
        });

        terrain_generator::generate_image(width, height, &terrain.height_map, &terrain.biomes_map)
            .save(&format!("/tmp/res/image-{}.png", i))
            .unwrap();

        for (j, r) in terrain.resources_map.iter().enumerate() {
            terrain_generator::save_gradient_as_image(
                width,
                height,
                r,
                &format!("/tmp/res/resource-{}-{}-{}-{}.png", i, j, p.amount, p.disp),
            );
        }
    }

    // println!("{:?}", galaxy);
}
