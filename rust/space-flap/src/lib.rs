#![allow(unused)]

extern crate core;

use itertools::Itertools;
use space_domain::game::astrobody::{AstroBodies, AstroBody, OrbitalPos};
use space_domain::game::events;
use space_domain::game::extractables::Extractable;
use space_domain::game::factory::Factory;
use space_domain::game::fleets::Fleet;
use space_domain::game::loader::{Loader, RandomMapCfg};
use space_domain::game::locations::{Location, Locations};
use space_domain::game::objects::ObjId;
use space_domain::game::order::{Order, Orders};
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::Game;
use space_domain::utils::{Position, V2_ZERO};
use specs::prelude::*;
use std::cell::RefCell;
use std::os::linux::raw::stat;
use std::path::PathBuf;
use std::rc::Rc;

pub type Id = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventKind {
    Add,
    Move,
    Jump,
    Dock,
    Undock,
}

#[derive(Clone, Debug)]
pub struct ObjKind {
    fleet: bool,
    jump: bool,
    station: bool,
    asteroid: bool,
    astro: bool,
}

#[derive(Clone, Debug)]
pub struct ObjOrbitData {
    radius: f32,
    parent_pos: Position,
}

impl ObjOrbitData {
    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_parent_pos(&self) -> (f32, f32) {
        (self.parent_pos.x, self.parent_pos.y)
    }
}

#[derive(Clone, Debug)]
pub struct ObjData {
    id: Entity,
    coords: Position,
    sector_id: Entity,
    docked: Option<Entity>,
    kind: ObjKind,
    orbit: Option<ObjOrbitData>,
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

    pub fn get_orbit(&self) -> Option<ObjOrbitData> {
        self.orbit.clone()
    }

    pub fn is_fleet(&self) -> bool {
        self.kind.fleet
    }

    pub fn is_station(&self) -> bool {
        self.kind.station
    }

    pub fn is_asteroid(&self) -> bool {
        self.kind.asteroid
    }

    pub fn is_jump(&self) -> bool {
        self.kind.jump
    }

    pub fn is_astro(&self) -> bool {
        self.kind.astro
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
        let mut size = 50;
        let mut fleets = 100;

        for mut pair in &args.iter().chunks(2) {
            let k = pair.next().unwrap();
            let v = pair.next().unwrap();
            log::info!("checking {}/{}", k, v);
            match k.as_str() {
                "--size" => match v.parse::<usize>() {
                    Ok(v) => {
                        log::info!("set size to {}", v);
                        size = v
                    }
                    Err(e) => {
                        log::warn!("invalid argument {}={}", k, v);
                    }
                },
                "--fleets" => match v.parse::<usize>() {
                    Ok(v) => {
                        log::info!("set fleet to {}", v);
                        fleets = v
                    }
                    Err(e) => {
                        log::warn!("invalid argument {}={}", k, v);
                    }
                },
                _ => log::warn!("unknown argument {}={}", k, v),
            }
        }

        let universe_cfg = space_domain::space_galaxy::system_generator::new_config_from_file(
            &PathBuf::from("data/system_generator.conf"),
        );

        let mut game = Game::new();
        Loader::load_random(
            &mut game,
            &RandomMapCfg {
                size: size,
                seed: 0,
                ships: fleets,
                universe_cfg,
            },
        );

        SpaceGame {
            game: Rc::new(RefCell::new(game)),
        }
    }

    pub fn list_at_sector(&self, sector_id: Id) -> Vec<Id> {
        let g = self.game.borrow();

        let entities = g.world.entities();

        let e_sector = decode_entity_and_get(&g, sector_id);

        let locations = g.world.read_storage::<Location>();
        let mut result = vec![];
        for (e, l) in (&entities, &locations).join() {
            if l.get_sector_id() == e_sector {
                result.push(encode_entity(e));
            }
        }
        result
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
        let fleets = g.world.read_storage::<Fleet>();

        let mut r = vec![];
        for (e, flt, st, j, l) in (
            &entities,
            &fleets,
            (&stations).maybe(),
            (&jumps).maybe(),
            &locations,
        )
            .join()
        {
            let ls = Locations::resolve_space_position_from(&locations, l)
                .expect("fail to find location");

            let kind = ObjKind {
                fleet: true,
                jump: j.is_some(),
                station: st.is_some(),
                asteroid: false,
                astro: false,
            };

            r.push(ObjData {
                id: e,
                coords: ls.pos,
                sector_id: ls.sector_id,
                docked: l.as_docked(),
                kind: kind,
                orbit: None,
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

    pub fn take_events(&mut self) -> Vec<EventData> {
        let events = self
            .game
            .borrow_mut()
            .world
            .fetch_mut::<events::Events>()
            .take();
        events.into_iter().map(|i| EventData { event: i }).collect()
    }

    pub fn get_obj(&self, id: Id) -> Option<ObjData> {
        let g = self.game.borrow();

        let entities = g.world.entities();

        let e = decode_entity_and_get(&g, id)?;

        let locations = g.world.read_storage::<Location>();
        let stations = g.world.read_storage::<Station>();
        let extractables = g.world.read_storage::<Extractable>();
        let jumps = g.world.read_storage::<Jump>();
        let fleets = g.world.read_storage::<Fleet>();
        let astros = g.world.read_storage::<AstroBody>();
        let orbits = g.world.read_storage::<OrbitalPos>();

        let flt = (&fleets).get(e);
        let loc = (&locations).get(e)?;
        let ext = (&extractables).get(e);
        let st = (&stations).get(e);
        let ls = Locations::resolve_space_position_from(&locations, loc)?;
        let jp = (&jumps).get(e);
        let ab = astros.get(e);
        let orb = orbits.get(e);

        let kind = ObjKind {
            fleet: flt.is_some(),
            jump: jp.is_some(),
            station: st.is_some(),
            asteroid: ext.is_some(),
            astro: ab.is_some(),
        };

        let orbit_data = orb.map(|o| {
            let pos = loc.get_pos().unwrap();
            let local_pos = o.compute_local_pos();
            let parent_pos = pos.sub(&local_pos);

            ObjOrbitData {
                radius: o.distance,
                parent_pos: parent_pos,
            }
        });

        let obj = ObjData {
            id: e,
            coords: ls.pos,
            sector_id: ls.sector_id,
            docked: loc.as_docked(),
            kind: kind,
            orbit: orbit_data,
        };

        Some(obj)
    }
}

pub struct EventData {
    event: events::Event,
}

impl EventData {
    pub fn get_id(&self) -> Id {
        encode_entity(self.event.id)
    }

    pub fn get_kind(&self) -> EventKind {
        match self.event.kind {
            events::EventKind::Add => EventKind::Add,
            events::EventKind::Move => EventKind::Move,
            events::EventKind::Jump => EventKind::Jump,
            events::EventKind::Dock => EventKind::Dock,
            events::EventKind::Undock => EventKind::Undock,
        }
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

fn decode_entity_and_get(g: &Game, id: Id) -> Option<Entity> {
    let (eid, egen) = decode_entity(id);
    let entities = g.world.entities();
    let e = entities.entity(eid);
    if egen == e.gen().id() {
        Some(e)
    } else {
        log::warn!(
            "get_obj for {}/{} fail, entity has gen {}",
            eid,
            egen,
            e.gen().id()
        );
        return None;
    }
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
