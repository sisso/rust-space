use commons::math::V2;
use commons::prob::{self, select_weighted, RDistrib, Weighted};
use commons::tree::Tree;
use commons::unwrap_or_continue;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn new_config_from_file(path: &Path) -> UniverseCfg {
    let wrapper: UniverseCfgWrapper =
        commons::hocon::load_file(path).expect("fail to read universe config file file");
    wrapper.system_generator
}

pub fn new_config_from_str(data: &str) -> UniverseCfg {
    let wrapper: UniverseCfgWrapper =
        commons::hocon::load_str(data).expect("invalid universe config file");
    wrapper.system_generator
}

pub type Deg = f32;
const RETRIES: usize = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstroProb {
    pub count_prob: RDistrib,
    pub distance_prob: RDistrib,
    pub rotation_speed_prob: RDistrib,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniverseCfgWrapper {
    system_generator: UniverseCfg,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniverseCfg {
    pub planets_prob: AstroProb,
    pub moons_prob: AstroProb,
    pub asteroids_prob: AstroProb,
    pub biomes_kinds: Vec<Weighted<String>>,
    pub atm_kinds: Vec<Weighted<String>>,
    pub ocean_kinds: Vec<Weighted<String>>,
    pub gravity_force: RDistrib,
    pub star_size: RDistrib,
    pub planet_size: RDistrib,
    pub asteroid_size: RDistrib,
    pub star_kinds: Vec<Weighted<String>>,
    pub resources: Vec<Resource>,
    pub asteroid_atm: String,
    pub asteroid_biome: String,
    pub system_resources_max: usize,
    pub system_resources_amount: RDistrib,
    pub system_distance_padding: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct System {
    pub coords: V2,
    pub bodies: Vec<SpaceBody>,
}

impl System {
    pub fn get_tree(&self) -> Tree<usize> {
        let mut tree = commons::tree::Tree::new();
        for b in self.bodies.iter() {
            if b.index == 0 && b.parent == 0 {
                continue;
            }
            tree.insert(b.index, b.parent);
        }

        tree
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub atmosphere: String,
    pub gravity: f32,
    pub biome: String,
    pub ocean: String,
    pub resources: Vec<BodyResource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BodyDesc {
    Star { kind: String },
    AsteroidField { resources: Vec<BodyResource> },
    Planet(Planet),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpaceBody {
    pub index: usize,
    pub parent: usize,
    pub distance: f32,
    pub angle: Deg,
    pub speed: f32,
    pub size: f32,
    pub desc: BodyDesc,
}

pub struct GenerateParams {}

#[derive(Debug)]
pub enum GenerateError {
    Generic(String),
}

pub struct PlanetSubCfg {
    pub max_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyResource {
    pub resource: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub kind: String,
    pub prob: f32,
    pub always: Vec<String>,
    pub require: Vec<String>,
    pub forbidden: Vec<String>,
}

#[derive(Clone)]
pub struct IdGen {
    pub v: usize,
}

impl IdGen {
    pub fn next(&mut self) -> usize {
        let v = self.v;
        self.v += 1;
        v
    }
}

pub fn new_system(cfg: &UniverseCfg, seed: u64) -> System {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
    let rng = &mut rng;

    let mut igen = IdGen { v: 0 };

    let star_kind = prob::select_weighted(rng, &cfg.star_kinds).unwrap();

    let star_i = igen.next();
    let star = SpaceBody {
        index: star_i,
        parent: star_i,
        distance: 0.0,
        angle: 0.0,
        speed: 0.0,
        size: cfg.star_size.next(rng),
        desc: BodyDesc::Star {
            kind: star_kind.clone(),
        },
    };

    let mut bodies = vec![star];
    let mut aabb: commons::lineboundbox::LineBoundBox = Default::default();

    let num_planets = cfg.planets_prob.count_prob.next_int(rng);
    for _ in 0..num_planets {
        for _ in 0..RETRIES {
            let mut local_igen = igen.clone();
            let system_bodies = new_planet(cfg, rng, &mut local_igen, star_i);
            let (min, max) = compute_bounds(&system_bodies);
            if aabb.add(
                min - cfg.system_distance_padding,
                max + cfg.system_distance_padding,
            ) {
                bodies.extend(system_bodies);
                igen = local_igen;
                break;
            }
        }
    }

    let num_asteroids = cfg.asteroids_prob.count_prob.next_int(rng);
    for _ in 0..num_asteroids {
        for _ in 0..RETRIES {
            let mut local_igen = igen.clone();
            let asteroids = new_asteroid(cfg, rng, &mut local_igen, star_i);
            let (min, max) = compute_bounds(&asteroids);
            if aabb.add(
                min - cfg.system_distance_padding,
                max + cfg.system_distance_padding,
            ) {
                bodies.extend(asteroids);
                igen = local_igen;
                break;
            }
        }
    }

    System {
        coords: V2::new(0., 0.),
        bodies: bodies,
    }
}

fn compute_bounds(bodies: &Vec<SpaceBody>) -> (f32, f32) {
    let root = bodies[0].distance;
    let mut min = root;
    let mut max = root;

    for b in &bodies[1..] {
        min = min.min(root - b.distance);
        max = max.max(root + b.distance);
    }

    (min, max)
}

fn new_planet(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    igen: &mut IdGen,
    parent: usize,
) -> Vec<SpaceBody> {
    let planet_i = igen.next();
    let distance = cfg.planets_prob.distance_prob.next(rng);
    let atm = prob::select_weighted(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select_weighted(rng, &cfg.biomes_kinds)
        .unwrap()
        .clone();
    let ocean = prob::select_weighted(rng, &cfg.ocean_kinds)
        .unwrap()
        .clone();
    let resources = generate_body_resources(&cfg, rng, &atm, &biome, &ocean);
    let angle = prob::RDistrib::MinMax(0., 360.).next(rng);
    let speed = cfg.planets_prob.rotation_speed_prob.next(rng);

    let planet = SpaceBody {
        index: planet_i,
        parent,
        distance,
        angle,
        speed,
        size: cfg.planet_size.next(rng),
        desc: BodyDesc::Planet(Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            resources: resources,
        }),
    };

    let mut bodies = vec![planet];

    let num_m = cfg.moons_prob.count_prob.next_int(rng);

    for _ in 0..num_m {
        let body = new_moon(
            cfg,
            &PlanetSubCfg {
                max_distance: distance * 0.5,
            },
            rng,
            igen,
            planet_i,
        );
        bodies.extend(body);
    }

    bodies
}

fn new_moon(
    cfg: &UniverseCfg,
    sub_cfg: &PlanetSubCfg,
    rng: &mut StdRng,
    igen: &mut IdGen,
    parent: usize,
) -> Vec<SpaceBody> {
    let mut distance = cfg.moons_prob.distance_prob.next(rng);
    for _ in 0..RETRIES {
        if distance <= sub_cfg.max_distance {
            break;
        }
        distance = cfg.moons_prob.distance_prob.next(rng);
    }

    let planet_i = igen.next();
    let atm = prob::select_weighted(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select_weighted(rng, &cfg.biomes_kinds)
        .unwrap()
        .clone();
    let ocean = prob::select_weighted(rng, &cfg.ocean_kinds)
        .unwrap()
        .clone();
    let resources = generate_body_resources(&cfg, rng, &atm, &biome, &ocean);
    let angle = prob::RDistrib::MinMax(0., 360.).next(rng);
    let speed = cfg.moons_prob.rotation_speed_prob.next(rng);

    let moon = SpaceBody {
        index: planet_i,
        parent,
        distance,
        angle,
        speed,
        size: cfg.planet_size.next(rng),
        desc: BodyDesc::Planet(Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            resources,
        }),
    };

    vec![moon]
}

fn new_asteroid(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    id_gen: &mut IdGen,
    parent: usize,
) -> Vec<SpaceBody> {
    let distance = cfg.asteroids_prob.distance_prob.next(rng);

    let id = id_gen.next();
    let atm = prob::select_weighted(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select_weighted(rng, &cfg.biomes_kinds)
        .unwrap()
        .clone();
    let ocean = prob::select_weighted(rng, &cfg.ocean_kinds)
        .unwrap()
        .clone();
    let resources = generate_body_resources(&cfg, rng, &atm, &biome, &ocean);
    let angle = prob::RDistrib::MinMax(0., 360.).next(rng);
    let speed = cfg.asteroids_prob.rotation_speed_prob.next(rng);

    let asteroid = SpaceBody {
        index: id,
        parent,
        distance,
        angle,
        speed,
        size: cfg.asteroid_size.next(rng),
        desc: BodyDesc::AsteroidField { resources },
    };

    vec![asteroid]
}

struct FilteredResources<'a> {
    must: Vec<&'a Resource>,
    candidate: Vec<&'a Resource>,
}

fn filter_resources<'a>(
    resources: &'a Vec<Resource>,
    _atm: &str,
    biome: &str,
    _ocean: &str,
) -> FilteredResources<'a> {
    let mut fr = FilteredResources {
        must: vec![],
        candidate: vec![],
    };

    for resource in resources {
        let is_forbidden = resource
            .forbidden
            .iter()
            .find(|n| n.as_str().eq_ignore_ascii_case(biome))
            .is_some();

        if is_forbidden {
            continue;
        }

        let must_have = resource
            .always
            .iter()
            .find(|n| n.as_str().eq_ignore_ascii_case(biome))
            .is_some();

        if must_have {
            fr.must.push(resource);
            continue;
        }

        if resource.require.is_empty() {
            fr.candidate.push(resource);
            continue;
        }

        let has_biome = resource
            .require
            .iter()
            .find(|n| n.as_str().eq_ignore_ascii_case(biome))
            .is_some();

        if has_biome {
            fr.candidate.push(resource);
        }
    }

    fr
}

fn generate_body_resources(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    atm: &str,
    biome: &str,
    ocean: &str,
) -> Vec<BodyResource> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(rng.gen());
    let rng = &mut rng;

    let fr = filter_resources(&cfg.resources, atm, biome, ocean);

    let mut resources = vec![];
    for resource in fr.must {
        resources.push(BodyResource {
            resource: resource.kind.clone(),
            amount: 1.0,
        });
    }

    let mut candidates: Vec<Weighted<&Resource>> = vec![];
    for resource in fr.candidate {
        candidates.push(Weighted {
            prob: resource.prob,
            value: resource,
        })
    }

    for _ in resources.len()..(cfg.system_resources_max as usize) {
        let selected = unwrap_or_continue!(select_weighted(rng, &candidates));

        let amount = cfg.system_resources_amount.next_positive(rng);
        if amount <= 0.0 {
            continue;
        }

        // check if resource was already selected and sum with the amount
        match resources
            .iter_mut()
            .find(|i| i.resource.as_str() == selected.kind)
        {
            Some(found) => found.amount += amount,
            None => resources.push(BodyResource {
                resource: selected.kind.clone(),
                amount: amount,
            }),
        }
    }

    resources
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let cfg = new_config_from_file(
            std::path::PathBuf::from("../data/system_generator.conf").as_path(),
        );
        let system = new_system(&cfg, 0);
        assert!(0 < system.bodies.len());
    }
}
