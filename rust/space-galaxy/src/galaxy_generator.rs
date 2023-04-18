use commons::grid::Grid;
use commons::math::{P2, P2I, V2, V2I};
use rand::prelude::*;
use std::collections::HashSet;
use std::ops::Sub;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub seed: u64,
    pub size: i32,
}

#[derive(Debug, Clone)]
pub struct Galaxy {
    pub cfg: Cfg,
    pub sectors: commons::grid::Grid<Sector>,
    pub jumps: Vec<Jump>,
}

type SectorId = usize;

#[derive(Debug, Clone)]
pub struct Sector {
    pub id: SectorId,
    pub coords: P2I,
}

#[derive(Debug, Clone)]
pub struct Jump {
    pub sector_a: SectorId,
    pub pos_a: P2,
    pub sector_b: SectorId,
    pub pos_b: P2,
}

impl Galaxy {
    pub fn new(cfg: Cfg) -> Galaxy {
        let (sectors, jumps) = generate_random_map(cfg.size, cfg.seed);
        Galaxy {
            cfg,
            sectors,
            jumps,
        }
    }
}

fn generate_random_map(size: i32, seed: u64) -> (Grid<Sector>, Vec<Jump>) {
    log::debug!("generating random map with seed {}", seed);

    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    // generate galaxy grid
    let rgcfg = commons::random_grid::RandomGridCfg {
        width: size as usize,
        height: size as usize,
        portal_prob: 0.5,
        deep_levels: 1,
    };

    let grids = commons::random_grid::RandomGrid::new(&rgcfg, &mut rng);
    log::trace!("{:?}", grids);
    let rgrid = &grids.levels[0];
    let mut sectors = vec![];

    for i in 0..rgrid.len() {
        // create sectors
        let coords = rgrid.get_coords(i);
        let sector = Sector {
            id: i,
            coords: P2I::new(coords.0 as i32, coords.1 as i32),
        };
        sectors.push(sector);
    }

    let grid = commons::grid::Grid {
        width: size,
        height: size,
        list: sectors,
    };
    // add portals
    let mut jumps = vec![];
    let mut cached: HashSet<(usize, usize)> = Default::default();

    for sector_a in 0..rgrid.len() {
        for sector_b in rgrid.neighbors_connected(sector_a) {
            if !cached.insert((sector_a, sector_b)) {
                continue;
            }

            if !cached.insert((sector_b, sector_a)) {
                continue;
            }

            let coor_a: V2I = rgrid.get_coords_slice_i32(sector_a).into();
            let coor_b: V2I = rgrid.get_coords_slice_i32(sector_b).into();
            let delta = coor_b.sub(coor_a);

            fn get_pos<R: rand::Rng>(rng: &mut R, delta: i32) -> f32 {
                if delta == 0 {
                    rng.gen_range(0.0..6.0) - 3.0
                } else {
                    rng.gen_range(4.0..5.0) * delta as f32
                }
            }

            let pos_a = V2::new(get_pos(&mut rng, delta.x), get_pos(&mut rng, delta.y));
            let pos_b = V2::new(get_pos(&mut rng, -delta.x), get_pos(&mut rng, -delta.y));

            jumps.push(Jump {
                sector_a,
                pos_a,
                sector_b,
                pos_b,
            });
        }
    }

    (grid, jumps)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let g = Galaxy::new(Cfg { seed: 0, size: 2 });

        assert_eq!(4, g.sectors.list.len());
        assert!(g.jumps.len() >= 3, "num jubs is {}", g.jumps.len());
        assert_eq!("", format!("{:?}", g));
    }
}
