#![allow(unused)]

use space_domain::game::loader::{Loader, RandomMapCfg};
use space_domain::game::sectors::Sector;
use space_domain::game::Game;
use specs::prelude::*;
use std::borrow::Borrow;

type Id = u64;

#[derive(Clone)]
pub struct SectorData {
    index: Id,
    coords: (f32, f32),
}

impl SectorData {
    pub fn new() -> Self {
        SectorData {
            index: 0,
            coords: (0.0, 0.0),
        }
    }

    pub fn index(&self) -> Id {
        self.index
    }
    pub fn coords(&self) -> (f32, f32) {
        self.coords.clone()
    }
}

pub struct SpaceGame {
    game: Game,
}

impl SpaceGame {
    pub fn new(args: Vec<String>) -> Self {
        let mut game = Game::new();
        Loader::load_random(
            &mut game,
            &RandomMapCfg {
                size: 50,
                seed: 50,
                ships: 100,
            },
        );

        SpaceGame { game }
    }

    pub fn get_sectors(&self) -> Vec<SectorData> {
        let entities = self.game.world.entities();
        let sectors = self.game.world.read_storage::<Sector>();

        let mut r = vec![];
        for (e, s) in (&entities, &sectors).join() {
            r.push(SectorData {
                index: encode_entity(e),
                coords: (s.coords.x, s.coords.y),
            });
        }
        r
    }

    pub fn get_sector(&self, index: Id) -> SectorData {
        // let ss = self.game.world.read_storage::<Sector>();
        // let id = self.game.world.entities().borrow().entity(index);
        // let sector = ss.borrow().get(id).expect("sector by index not found");
        // SectorData {
        //     index: index,
        //     coords: (sector.coords.x, sector.coords.y),
        // }
        todo!()
    }
}

fn encode_entity(entity: Entity) -> u64 {
    let high: u32 = entity.id();
    let low: i32 = entity.gen().id();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    return encoded;
}

fn decode_entity(value: u64) -> (u32, i32) {
    let high = (value >> 32) as u32;
    let low = (value & 0xffffffff) as i32;
    (high, low)
}

include!(concat!(env!("OUT_DIR"), "/glue.rs"));

#[cfg(test)]
mod test {
    use super::*;
    use specs::world::Generation;
    use std::num::NonZeroI32;

    #[test]
    fn test1() {}

    #[test]
    fn test_encode_decode_entity() {
        let mut w = World::new();
        for _ in 0..100 {
            w.create_entity().build();
        }

        for _ in 0..9 {
            let e = w.create_entity().build();
            w.delete_entity(e).unwrap();
        }

        let e = w.create_entity().build();
        assert_eq!(100, e.id());
        assert_eq!(10, e.gen().id());

        let v = encode_entity(e);
        let (id, gen) = decode_entity(v);

        assert_eq!(e.id(), id);
        assert_eq!(100, id);
        assert_eq!(e.gen().id(), gen);
        assert_eq!(10, gen);
    }
}
