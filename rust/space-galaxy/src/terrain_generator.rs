/*
    Copied from https://github.com/Mapet13/terrain-generator-2d MIT
*/
use opensimplex_noise_rs::OpenSimplexNoise;

use image::{ImageBuffer, Rgb};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Terrain {
    pub width: i32,
    pub height: i32,
    pub biomes_map: Vec<f32>,
    pub height_map: Vec<f32>,
    pub resources_map: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Shape {
    Plain,
    Island,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ResCfg {
    // 0-1
    pub amount: f32,
    // 0-1
    pub disp: f32,
}

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub width: i32,
    pub height: i32,
    pub seed: u64,
    pub heighmap_scale: f64,
    pub biomemap_scale: f64,
    pub shape: Shape,
    pub resources: Vec<ResCfg>,
}

impl Cfg {
    pub fn total_indexes(&self) -> i32 {
        self.width * self.height
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg {
            width: 512,
            height: 512,
            seed: 0,
            heighmap_scale: 0.004,
            biomemap_scale: 0.007,
            shape: Shape::Plain,
            resources: vec![],
        }
    }
}

pub fn generate_terrain(cfg: &Cfg) -> Terrain {
    // let mut rng = StdRng::seed_from_u64(cfg.seed);

    let (heights, biomes) = generate_maps(cfg);
    let resources_maps = cfg
        .resources
        .iter()
        .map(|r| {
            log::debug!("generate resource {:?}", r);
            generate_resource(cfg.width, cfg.height, cfg.seed, r.disp, r.amount)
        })
        .collect();

    Terrain {
        width: cfg.width,
        height: cfg.height,
        biomes_map: biomes,
        height_map: heights,
        resources_map: resources_maps,
    }
}

fn sum_octaves(
    num_iterations: i32,
    point: (i32, i32),
    persistence: f64,
    scale: f64,
    low: f64,
    high: f64,
    noise_fn: impl Fn(f64, f64) -> f64,
) -> f64 {
    let mut max_amp = 0.0;
    let mut amp = 1.0;
    let mut freq = scale;
    let mut noise = 0.0;

    for _ in 0..num_iterations {
        noise += noise_fn(point.0 as f64 * freq, point.1 as f64 * freq) * amp;
        max_amp += amp;
        amp *= persistence;
        freq *= 2.0;
    }

    (noise / max_amp) * (high - low) / 2.0 + (high + low) / 2.0
}

pub fn generate_plain_gradient(pixels: i32) -> Vec<f32> {
    let gradient: Vec<f32> = vec![0.0; pixels as usize];
    gradient
}

pub fn generate_island_gradient(w: i32, h: i32) -> Vec<f32> {
    let mut gradient: Vec<f32> = vec![0.0; (w * h) as usize];

    for x in 0..w {
        for y in 0..h {
            let mut color_value: f32;

            let a = if x > (w / 2) { w - x } else { x };

            let b = if y > h / 2 { h - y } else { y };

            let smaller = std::cmp::min(a, b) as f32;
            color_value = smaller / (w as f32 / 2.0);

            color_value = 1.0 - color_value;
            color_value = color_value * color_value;

            gradient[get_id_from_pos(h, x, y)] = match color_value - 0.1 {
                x if x > 1.0 => 1.0,
                x if x < 0.0 => 0.0,
                x => x,
            };
        }
    }

    gradient
}

pub fn generate_maps(cfg: &Cfg) -> (Vec<f32>, Vec<f32>) {
    let gradient = match &cfg.shape {
        Shape::Plain => generate_plain_gradient(cfg.width * cfg.height),
        Shape::Island => generate_island_gradient(cfg.width, cfg.height),
    };

    let mut rng = StdRng::seed_from_u64(cfg.seed);

    let mut height_map = generate_noise_map(
        cfg.width,
        cfg.height,
        rng.gen_range(0..i64::MAX),
        cfg.heighmap_scale,
    );
    let mut biome_map = generate_noise_map(
        cfg.width,
        cfg.height,
        rng.gen_range(0..i64::MAX),
        cfg.biomemap_scale,
    );

    for x in 0..cfg.width {
        for y in 0..cfg.height {
            let index = get_id_from_pos(cfg.width, x, y);
            height_map[index] = height_map[index] * 1.1 - gradient[index] * 0.8;
            biome_map[index] = biome_map[index] - (0.1 - gradient[index]) * 0.4;
            if height_map[index] < 0.0 {
                height_map[index] = 0.0;
            }
            if biome_map[index] < 0.0 {
                biome_map[index] = 0.0;
            }
        }
    }

    (height_map, biome_map)
}

pub fn get_id_from_pos(width: i32, x: i32, y: i32) -> usize {
    (x + width * y) as usize
}

pub fn generate_noise_map(width: i32, height: i32, seed: i64, scale: f64) -> Vec<f32> {
    let noise_generator = OpenSimplexNoise::new(Some(seed));

    let mut map: Vec<f32> = vec![0.0; (width * height) as usize];
    for x in 0..width {
        for y in 0..height {
            let val = sum_octaves(16, (x, y), 0.5, scale, 0.0, 1.0, |x, y| {
                noise_generator.eval_2d(x, y)
            });

            map[get_id_from_pos(width, x, y)] = val as f32;
        }
    }
    map
}

enum Biomes {
    Grass,
    DeepWater,
    Water,
    Dirt,
    Sand,
    WetSand,
    DarkForest,
    HighDarkForest,
    LightForest,
    Mountain,
    HighMountain,
    Snow,
}

fn get_biome_color(biome: Biomes) -> [u8; 3] {
    match biome {
        Biomes::Grass => [120, 157, 80],
        Biomes::Water => [9, 82, 198],
        Biomes::DeepWater => [0, 62, 178],
        Biomes::Dirt => [114, 98, 49],
        Biomes::Sand => [194, 178, 128],
        Biomes::WetSand => [164, 148, 99],
        Biomes::DarkForest => [60, 97, 20],
        Biomes::HighDarkForest => [40, 77, 0],
        Biomes::LightForest => [85, 122, 45],
        Biomes::Mountain => [140, 142, 123],
        Biomes::HighMountain => [160, 162, 143],
        Biomes::Snow => [235, 235, 235],
    }
}

pub fn generate_image(
    w: i32,
    h: i32,
    height_map: &[f32],
    biome_map: &[f32],
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w as u32, h as u32);

    for x in 0..w {
        for y in 0..h {
            let height = height_map[get_id_from_pos(w, x, y)];
            let moisture = biome_map[get_id_from_pos(w, x, y)];

            let biome = match (height, moisture) {
                (a, _) if a < 0.39 => Biomes::DeepWater,
                (a, _) if a < 0.42 => Biomes::Water,
                (a, b) if a < 0.46 && b < 0.57 => Biomes::Sand,
                (a, b) if a < 0.47 && b < 0.6 => Biomes::WetSand,
                (a, b) if a < 0.47 && b >= 0.6 => Biomes::Dirt,
                (a, b) if a > 0.54 && b < 0.43 && a < 0.62 => Biomes::Grass,
                (a, b) if a < 0.62 && b >= 0.58 => Biomes::HighDarkForest,
                (a, b) if a < 0.62 && b >= 0.49 => Biomes::DarkForest,
                (a, _) if a >= 0.79 => Biomes::Snow,
                (a, _) if a >= 0.74 => Biomes::HighMountain,
                (a, b) if a >= 0.68 && b >= 0.10 => Biomes::Mountain,
                _ => Biomes::LightForest,
            };

            let color = get_biome_color(biome);
            let pixel = image.get_pixel_mut(x as u32, y as u32);
            *pixel = image::Rgb(color);
        }
    }

    image
}

pub fn gradient_to_rgb(value: f32) -> u8 {
    (value * 256.0).max(0.0).min(256.0).round() as u8
}

pub fn save_gradient_as_image(w: i32, h: i32, gradient: &Vec<f32>, file_name: &str) {
    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w as u32, h as u32);
    for x in 0..w {
        for y in 0..h {
            let v = gradient_to_rgb(gradient[get_id_from_pos(w, x, y)]);
            let color = [v, 0, 0];
            let pixel = image.get_pixel_mut(x as u32, y as u32);
            *pixel = image::Rgb(color);
        }
    }
    image.save(file_name).unwrap();
}

pub fn save_image(w: i32, h: i32, img: Vec<u8>, file_name: &str) {
    let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(w as u32, h as u32, img).unwrap();
    image.save(file_name).unwrap();
}

/// scale = 0.004 amount = 0.9
/// dist: 0.0 means all quantity is concentrated in a single point, 1.0 means it is spread in every
///       pixel
/// quantity: for the pixels tha contain resource,
pub fn generate_resource(w: i32, h: i32, seed: u64, dist: f32, quantity: f32) -> Vec<f32> {
    fn normalize(img: &mut Vec<f32>) {
        let mut min: f32 = 1.0;
        let mut max: f32 = 0.0;
        for i in img.iter() {
            min = min.min(*i);
            max = max.max(*i);
        }

        let delta = max - min;
        for i in img.iter_mut() {
            let ni = (*i - min) / delta;
            *i = ni;
        }
    }

    let scale = commons::math::lerp(0.01, 0.02, dist);
    let mut res_map = generate_noise_map(w, h, seed as i64, scale as f64);

    // normalize noise between 0-1
    normalize(&mut res_map);

    // filter values by availability
    let selector = 1.0 - dist;
    for i in res_map.iter_mut() {
        if *i >= selector {
            // normalize between 0 1
            *i = quantity;
        } else {
            *i = 0.0;
        }
    }

    res_map
}
