#![allow(unused)]

use space_domain::game::extractables::Extractable;
use space_domain::game::factory::Factory;
use space_domain::game::loader::{Loader, RandomMapCfg};
use space_domain::game::locations::{Location, Locations};
use space_domain::game::order::{Order, Orders};
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::Game;
use space_domain::utils::{Position, V2_ZERO};
use specs::prelude::*;
use std::cell::RefCell;
use std::os::linux::raw::stat;
use std::rc::Rc;

type Id = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjKind {
    Fleet,
    Asteroid,
    Station,
}

#[derive(Clone, Debug)]
pub struct ObjData {
    id: Entity,
    coords: Position,
    sector_id: Entity,
    docked: Option<Entity>,
    kind: ObjKind,
}

impl ObjData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.id)
    }

    pub fn is_docked(&self) -> bool {
        self.docked.is_some()
    }

    pub fn get_docked_id(&self) -> Option<Id> {
        self.docked.map(|e| encode_entity(e))
    }

    pub fn get_sector_id(&self) -> Id {
        encode_entity(self.sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        (self.coords.x, self.coords.y)
    }

    pub fn get_kind(&self) -> ObjKind {
        self.kind
    }
}

#[derive(Clone, Debug)]
pub struct SectorData {
    id: Id,
    coords: (f32, f32),
}

#[derive(Clone)]
pub struct JumpData {
    entity: Entity,
    game: Rc<RefCell<Game>>,
}

impl JumpData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.entity)
    }

    pub fn get_sector_id(&self) -> Id {
        let g = self.game.borrow();
        let locations = g.world.read_storage::<Location>();
        let loc = Locations::resolve_space_position(&locations, self.entity);
        encode_entity(loc.unwrap().sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        let g = self.game.borrow();
        let locations = g.world.read_storage::<Location>();
        let loc = Locations::resolve_space_position(&locations, self.entity);
        let pos = loc.unwrap().pos;
        (pos.x, pos.y)
    }

    pub fn get_to_sector_id(&self) -> Id {
        let g = self.game.borrow();
        let jumps = g.world.read_storage::<Jump>();
        encode_entity((&jumps).get(self.entity).unwrap().target_sector_id)
    }

    pub fn get_to_coords(&self) -> (f32, f32) {
        let g = self.game.borrow();
        let jumps = g.world.read_storage::<Jump>();
        let pos = (&jumps).get(self.entity).unwrap().target_pos;
        (pos.x, pos.y)
    }
}

impl SectorData {
    pub fn get_id(&self) -> Id {
        self.id
    }
    pub fn get_coords(&self) -> (f32, f32) {
        self.coords.clone()
    }
}

pub struct SpaceGame {
    game: Rc<RefCell<Game>>,
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

        SpaceGame {
            game: Rc::new(RefCell::new(game)),
        }
    }

    pub fn get_sectors(&self) -> Vec<SectorData> {
        let g = self.game.borrow();

        let entities = g.world.entities();
        let sectors = g.world.read_storage::<Sector>();

        let mut r = vec![];
        for (e, s) in (&entities, &sectors).join() {
            r.push(SectorData {
                id: encode_entity(e),
                coords: (s.coords.x, s.coords.y),
            });
        }
        r
    }

    pub fn get_jumps(&self) -> Vec<JumpData> {
        let g = self.game.borrow();

        let entities = g.world.entities();
        let jumps = g.world.read_storage::<Jump>();

        let mut r = vec![];
        for (e, _) in (&entities, &jumps).join() {
            r.push(JumpData {
                entity: e,
                game: self.game.clone(),
            });
        }
        r
    }

    pub fn get_fleets(&self) -> Vec<ObjData> {
        let g = self.game.borrow();

        let entities = g.world.entities();
        let locations = g.world.read_storage::<Location>();
        let stations = g.world.read_storage::<Station>();
        let jumps = g.world.read_storage::<Jump>();
        let extractables = g.world.read_storage::<Extractable>();

        let mut r = vec![];
        for (e, st, ext, _, l) in (
            &entities,
            (&stations).maybe(),
            (&extractables).maybe(),
            !(&jumps),
            &locations,
        )
            .join()
        {
            let ls = Locations::resolve_space_position_from(&locations, l)
                .expect("fail to find location");

            let kind = if ext.is_some() {
                ObjKind::Asteroid
            } else if st.is_some() {
                ObjKind::Station
            } else {
                ObjKind::Fleet
            };

            r.push(ObjData {
                id: e,
                coords: ls.pos,
                sector_id: ls.sector_id,
                docked: l.as_docked(),
                kind: kind,
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

    pub fn update(&mut self, delta: f32) {
        self.game.borrow_mut().tick(delta.into());
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
    use space_domain::utils::{MIN_DISTANCE, MIN_DISTANCE_SQR, V2};
    use specs::world::Generation;
    use std::num::NonZeroI32;

    #[test]
    fn test1() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .init();

        let mut sg = SpaceGame::new(vec![]);
        let f1 = sg.get_fleets();

        for _ in 0..100 {
            sg.update(1.0);
        }

        let f2 = sg.get_fleets();
        let mut changed_pos = 0;

        for f in f1 {
            for f2 in &f2 {
                if f.id == f2.id {
                    let changed = V2::distance(&f.coords, &f2.coords) > MIN_DISTANCE;
                    if f.kind == ObjKind::Fleet && changed {
                        changed_pos += 1;
                    } else if f.kind == ObjKind::Station && changed {
                        panic!("station should not move on {:?}", f);
                    }
                }
            }
        }

        assert!(changed_pos > 0);
    }

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
