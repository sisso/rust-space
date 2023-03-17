#![allow(unused)]

extern crate core;

use commons::math::P2;
use itertools::{cloned, Itertools};
use space_domain::game::actions::{Action, ActionActive, Actions};
use space_domain::game::astrobody::{AstroBodies, AstroBody, OrbitalPos};
use space_domain::game::extractables::Extractable;
use space_domain::game::factory::Factory;
use space_domain::game::fleets::Fleet;
use space_domain::game::loader::Loader;
use space_domain::game::locations::{Location, LocationSpace, Locations};
use space_domain::game::navigations::{Navigation, NavigationMoveTo};
use space_domain::game::objects::ObjId;
use space_domain::game::order::{Order, Orders};
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::wares::{Cargo, WareId};
use space_domain::game::Game;
use space_domain::game::{events, scenery_random};
use space_domain::utils::TotalTime;
use specs::prelude::*;
use std::cell::RefCell;
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
    parent_pos: P2,
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
pub struct ObjCoords {
    location: LocationSpace,
    is_docked: bool,
}

impl ObjCoords {
    pub fn get_sector_id(&self) -> Id {
        encode_entity(self.location.sector_id)
    }

    pub fn get_coords(&self) -> (f32, f32) {
        (self.location.pos.x, self.location.pos.y)
    }

    pub fn is_docked(&self) -> bool {
        self.is_docked
    }
}

#[derive(Clone, Debug)]
pub struct ObjData {
    id: Entity,
    coords: P2,
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
pub enum ObjActionKind {
    Undock,
    Jump,
    Dock,
    MoveTo,
    MoveToTargetPos,
    Extract,
}

#[derive(Clone, Debug)]
pub struct ObjAction {
    action: Action,
}

impl ObjAction {
    pub fn get_kind(&self) -> ObjActionKind {
        match &self.action {
            Action::Undock => ObjActionKind::Undock,
            Action::Jump { .. } => ObjActionKind::Jump,
            Action::Dock { .. } => ObjActionKind::Dock,
            Action::MoveTo { .. } => ObjActionKind::MoveTo,
            Action::MoveToTargetPos { .. } => ObjActionKind::MoveToTargetPos,
            Action::Extract { .. } => ObjActionKind::Extract,
        }
    }

    pub fn get_target(&self) -> Option<Id> {
        match &self.action {
            Action::Jump { jump_id } => Some(encode_entity(*jump_id)),
            Action::Dock { target_id } => Some(encode_entity(*target_id)),
            Action::MoveToTargetPos { target_id, .. } => Some(encode_entity(*target_id)),
            Action::Extract { target_id, .. } => Some(encode_entity(*target_id)),
            _ => None,
        }
    }

    pub fn get_pos(&self) -> Option<(f32, f32)> {
        match &self.action {
            _ => None,
            Action::MoveTo { pos } => Some((pos.x, pos.y)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjDesc {
    id: Id,
    extractable: Option<Entity>,
    action: Option<Action>,
    nav_move_to: Option<NavigationMoveTo>,
    cargo: Option<Cargo>,
}

impl ObjDesc {
    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_action(&self) -> Option<ObjAction> {
        self.action.as_ref().map(|action| ObjAction {
            action: action.clone(),
        })
    }

    pub fn get_nav_move_to_target(&self) -> Option<Id> {
        self.nav_move_to
            .as_ref()
            .map(|i| encode_entity(i.target_id))
    }

    pub fn get_cargo(&self) -> Option<ObjCargo> {}

    // pub fn x(&self) {
    //     self.nav_move_to
    //         .map(|i| encode_entity(i.plan.path.iter().map(|j| j.clone())))
    // }
}

#[derive(Clone, Debug)]
pub struct ObjCargo {
    cargo: Cargo,
}

impl ObjCargo {
    fn volume_total(&self) -> f32 {
        self.cargo.get_current()
    }
    fn volume_max(&self) -> f32 {
        self.cargo.get_max()
    }
    fn wares_count(&self) -> usize {
        self.cargo.get_wares_ids().count()
    }
    fn get_ware(&self, index: usize) -> (Id, f32) {}
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
        scenery_random::load_random(
            &mut game,
            &scenery_random::RandomMapCfg {
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
            let parent_pos = locations.get(o.parent).and_then(|i| i.as_space()).unwrap();
            ObjOrbitData {
                radius: o.distance,
                parent_pos: parent_pos.pos,
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

    pub fn get_obj_desc(&self, id: Id) -> Option<ObjDesc> {
        let g = self.game.borrow();
        let entities = g.world.entities();
        let e = decode_entity_and_get(&g, id)?;

        let desc = ObjDesc {
            id: id,
            extractable: g
                .world
                .read_storage::<Extractable>()
                .get(e)
                .map(|ext| ext.ware_id),
            action: g
                .world
                .read_storage::<ActionActive>()
                .get(e)
                .map(|action| action.get_action().clone()),
            nav_move_to: g.world.read_storage::<NavigationMoveTo>().get(e).cloned(),
            cargo: g.world.read_storage::<Cargo>().get(e).cloned(),
        };

        Some(desc)
    }

    pub fn get_obj_coords(&self, id: Id) -> Option<ObjCoords> {
        let game = self.game.borrow();
        let e = decode_entity_and_get(&game, id)?;
        let locations = game.world.read_storage::<Location>();
        let loc = locations.get(e)?;
        let ls = Locations::resolve_space_position_from(&locations, loc)?;
        let is_docked = loc.get_pos().is_none();
        Some(ObjCoords {
            location: ls,
            is_docked,
        })
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

// real encoding of a entity
fn proper_encode_entity(entity: Entity) -> u64 {
    let high: u32 = entity.id();
    let low: i32 = entity.gen().id();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    return encoded;
}

// real decoding of a entity
fn proper_decode_entity(value: u64) -> (u32, i32) {
    let high = (value >> 32) as u32;
    let low = (value & 0xffffffff) as i32;
    (high, low)
}

// pretty but broken encode of entity
fn encode_entity(entity: Entity) -> u64 {
    let high = entity.id() as u64 * 1_000_000;
    let low = entity.gen().id() as u64;
    return high + low;
}

// pretty but broken decode of entity
fn decode_entity(value: u64) -> (u32, i32) {
    let high = value / 1_000_000;
    let low = value % 1_000_000;
    (high as u32, low as i32)
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
    fn test_v2_distance() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .try_init();

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
                    let changed = V2::distance(f.coords, f2.coords) > MIN_DISTANCE;
                    if f.kind.fleet && changed {
                        changed_pos += 1;
                    } else if f.kind.station && changed {
                        panic!("station should not move on {:?}", f);
                    }
                }
            }
        }

        assert!(changed_pos > 0);
    }

    #[test]
    fn test_proper_encode_decode_entity() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .try_init();

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

        let v = proper_encode_entity(e);
        log::info!("encoded {v}");
        let (id, gen) = proper_decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.id(), id);
        assert_eq!(100, id);
        assert_eq!(e.gen().id(), gen);
        assert_eq!(10, gen);
    }

    #[test]
    fn test_encode_decode_entity() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .try_init();

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
        log::info!("encoded {v}");
        let (id, gen) = decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.id(), id);
        assert_eq!(100, id);
        assert_eq!(e.gen().id(), gen);
        assert_eq!(10, gen);
    }
}
