use log::debug;
use rand::prelude::*;
use space_galaxy::{galaxy_generator, system_generator, terrain_generator};
use std::cmp::max;
use std::path::{Path, PathBuf};

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let seed = 0u64;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let width = 300;
    let height = 200;
    let params = vec![(10000.0), (1000.0), (100.0), (10.0), (0.1)];

    for (i, p) in params.iter().enumerate() {
        let terrain = terrain_generator::generate_terrain(&terrain_generator::Cfg {
            width: width,
            height: height,
            seed: rng.gen(),
            heighmap_scale: 0.004,
            biomemap_scale: 0.007,
            shape: terrain_generator::Shape::Plain,
            resources: vec![*p],
        });

        for (j, r) in terrain.resources_map.iter().enumerate() {
            terrain_generator::save_gradient_as_image(
                width,
                height,
                r,
                &format!("/tmp/res/resource-{}-{}.png", i, j),
            );
        }
    }

    // println!("{:?}", galaxy);
}
