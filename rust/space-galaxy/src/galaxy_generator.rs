use commons::grid::Grid;
use commons::math::{V2, V2I};
use rand::prelude::*;
use std::collections::HashSet;

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
    pub coords: V2I,
}

#[derive(Debug, Clone)]
pub struct Jump {
    pub sector_a: SectorId,
    pub pos_a: V2,
    pub sector_b: SectorId,
    pub pos_b: V2,
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

    fn sector_pos<R: rand::Rng>(rng: &mut R) -> V2 {
        V2::new(
            (rng.gen_range(0..10) - 5) as f32,
            (rng.gen_range(0..10) - 5) as f32,
        )
    }

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
            coords: V2I::new(coords.0 as i32, coords.1 as i32),
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

    for index in 0..rgrid.len() {
        for other in rgrid.neighbors_connected(index) {
            if !cached.insert((index, other)) {
                continue;
            }

            if !cached.insert((other, index)) {
                continue;
            }

            jumps.push(Jump {
                sector_a: index,
                pos_a: sector_pos(&mut rng),
                sector_b: other,
                pos_b: sector_pos(&mut rng),
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
