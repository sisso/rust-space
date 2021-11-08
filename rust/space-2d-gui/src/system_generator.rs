use commons::math::V2;
use commons::prob::{self, select_weighted, RDistrib, Weighted};
use commons::tree::Tree;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn new_config(path: &Path) -> UniverseCfg {
    let wrapper: UniverseCfgWrapper =
        commons::hocon::load_hocon_files(path).expect("fail to read file");
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
    // pub moons_moons_prob: AstroProb,
    pub asteroids_prob: AstroProb,
    pub biomes_kinds: Vec<Weighted<String>>,
    pub atm_kinds: Vec<Weighted<String>>,
    pub ocean_kinds: Vec<Weighted<String>>,
    pub gravity_force: RDistrib,
    pub star_size: RDistrib,
    pub planet_size: RDistrib,
    pub star_kinds: Vec<Weighted<String>>,
    pub resources: Vec<Resource>,
    pub system_resources_max: u32,
    pub system_resources_amount: RDistrib,
    pub system_distance_padding: f32,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum BodyDesc {
    Star {
        kind: String,
    },
    AsteroidField {
        resources: Vec<BodyResource>,
    },
    Planet {
        atmosphere: String,
        gravity: f32,
        biome: String,
        ocean: String,
        resources: Vec<BodyResource>,
    },
}

#[derive(Clone, Debug)]
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
    max_distance: f32,
}

#[derive(Debug, Clone)]
pub struct BodyResource {
    resource: String,
    amount: f32,
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

pub fn new_system(cfg: &UniverseCfg, rng: &mut StdRng) -> System {
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

    let num_bodies = cfg.planets_prob.count_prob.next_int(rng);
    for _ in 0..num_bodies {
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

/*
sytem
2 - planet
  1 - moon


[(1,3)]

--###

4 - planet
1 fail, 4-1 =3 and is in use
reject? retry?
 */
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
    let resources = generate_resources(&cfg, rng, &atm, &biome, &ocean);
    let angle = prob::RDistrib::MinMax(0., 360.).next(rng);
    let speed = cfg.planets_prob.rotation_speed_prob.next(rng);

    let planet = SpaceBody {
        index: planet_i,
        parent,
        distance,
        angle,
        speed,
        size: cfg.planet_size.next(rng),
        desc: BodyDesc::Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            resources: resources,
        },
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
    let resources = generate_resources(&cfg, rng, &atm, &biome, &ocean);
    let angle = prob::RDistrib::MinMax(0., 360.).next(rng);
    let speed = cfg.moons_prob.rotation_speed_prob.next(rng);

    let moon = SpaceBody {
        index: planet_i,
        parent,
        distance,
        angle,
        speed,
        size: cfg.planet_size.next(rng),
        desc: BodyDesc::Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            resources,
        },
    };

    vec![moon]
}

fn generate_resources(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    _atm: &str,
    biome: &str,
    _ocean: &str,
) -> Vec<BodyResource> {
    fn cmp(a: &str, b: &str) -> bool {
        a.eq_ignore_ascii_case(b)
    }

    let mut rng: StdRng = SeedableRng::seed_from_u64(rng.gen());
    let rng = &mut rng;
    let mut resources = vec![];
    let mut candidates: Vec<Weighted<&Resource>> = vec![];

    cfg.resources
        .iter()
        .flat_map(|r| {
            if r.forbidden
                .iter()
                .find(|n| cmp(n.as_str(), biome))
                .is_some()
            {
                None
            } else if r.always.iter().find(|n| cmp(n.as_str(), biome)).is_some() {
                resources.push(BodyResource {
                    resource: r.kind.to_string(),
                    amount: 1.0,
                });
                None
            } else if !r.require.is_empty()
                && r.require.iter().find(|n| cmp(n.as_str(), biome)).is_none()
            {
                None
            } else {
                Some(r)
            }
        })
        .for_each(|r| {
            candidates.push(Weighted {
                prob: r.prob,
                value: r,
            })
        });

    for _ in resources.len()..(cfg.system_resources_max as usize) {
        let selected = select_weighted(rng, &candidates);
        if selected.is_none() {
            continue;
        }
        let selected = selected.unwrap();

        if selected.kind == "none" {
            continue;
        }

        let amount = cfg.system_resources_amount.next_positive(rng);
        if amount <= 0.0 {
            continue;
        }

        match resources
            .iter_mut()
            .find(|i| i.resource.as_str() == selected.kind)
        {
            Some(found) => found.amount += cfg.system_resources_amount.next(rng),

            None => resources.push(BodyResource {
                resource: selected.kind.clone(),
                amount: amount,
            }),
        }
    }

    resources
}
