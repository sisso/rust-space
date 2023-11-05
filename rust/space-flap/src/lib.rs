#![allow(unused)]

extern crate core;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use itertools::{cloned, Itertools};
use specs::prelude::*;

use commons::math::P2;
pub use models::*;
use space_domain::game::actions::{Action, ActionActive, Actions};
use space_domain::game::astrobody::{AstroBodies, AstroBody, AstroBodyKind, OrbitalPos};
use space_domain::game::conf::BlueprintCode;
use space_domain::game::dock::Docking;
use space_domain::game::extractables::Extractable;
use space_domain::game::factory::Factory;
use space_domain::game::fleets::Fleet;
use space_domain::game::label::Label;
use space_domain::game::loader::Loader;
use space_domain::game::locations::{Location, LocationSpace, Locations};
use space_domain::game::navigations::{Navigation, NavigationMoveTo};
use space_domain::game::objects::ObjId;
use space_domain::game::order::TradeOrders;
use space_domain::game::prefab::Prefab;
use space_domain::game::production_cost::ProductionCost;
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::wares::{Cargo, Ware, WareId};
use space_domain::game::{events, scenery_random, shipyard};
use space_domain::game::{loader, Game};
use space_domain::utils::TotalTime;
use utils::*;

pub mod models;
mod utils;

// EOF models

pub struct SpaceGame {
    game: Rc<RefCell<Game>>,
}

impl SpaceGame {
    pub fn new(args: Vec<String>) -> Self {
        let mut size = 4;
        let mut fleets = 10;
        let mut stations = 4;
        let mut seed = 0;

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
                "--seed" => match v.parse::<u64>() {
                    Ok(v) => {
                        log::info!("set seed to {}", v);
                        seed = v
                    }
                    Err(e) => {
                        log::warn!("invalid argument {}={}", k, v);
                    }
                },
                _ => log::warn!("unknown argument {}={}", k, v),
            }
        }

        let system_generator_conf = include_str!("../../data/game.conf");
        let cfg = space_domain::game::conf::load_str(system_generator_conf)
            .expect("fail to read config file");

        let mut game = Game::new();
        loader::load_prefabs(&mut game.world, &cfg.prefabs);
        scenery_random::load_random(
            &mut game,
            &scenery_random::RandomMapCfg {
                size: size,
                seed: seed,
                fleets: fleets,
                universe_cfg: cfg.system_generator.unwrap(),
                initial_condition: scenery_random::InitialCondition::Minimal,
                params: cfg.params,
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
        let labels = g.world.read_storage::<Label>();

        let mut r = vec![];
        for (e, s, l) in (&entities, &sectors, &labels).join() {
            r.push(SectorData {
                id: encode_entity(e),
                coords: (s.coords.x, s.coords.y),
                label: l.label.clone(),
            });
        }
        r
    }

    pub fn list_jumps(&self) -> Vec<JumpData> {
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

    pub fn get_jump(&self, id: Id) -> Option<JumpData> {
        let g = self.game.borrow();
        let e = decode_entity_and_get(&g, id)?;
        let jumps = g.world.read_storage::<Jump>();
        let jump = jumps.get(e)?;
        Some(JumpData {
            entity: e,
            game: self.game.clone(),
        })
    }

    pub fn get_label(&self, id: Id) -> Option<String> {
        self.game
            .borrow()
            .world
            .read_storage::<Label>()
            .get(decode_entity_and_get(&self.game.borrow(), id)?)
            .map(|l| l.label.clone())
    }

    pub fn get_fleets(&self) -> Vec<ObjData> {
        let g = self.game.borrow();

        let entities = g.world.entities();
        let locations = g.world.read_storage::<Location>();
        let stations = g.world.read_storage::<Station>();
        let jumps = g.world.read_storage::<Jump>();
        let fleets = g.world.read_storage::<Fleet>();
        let orders = g.world.read_storage::<TradeOrders>();

        let mut r = vec![];
        for (e, flt, st, j, l, o) in (
            &entities,
            &fleets,
            (&stations).maybe(),
            (&jumps).maybe(),
            &locations,
            &orders,
        )
            .join()
        {
            let ls = match Locations::resolve_space_position_from(&locations, l) {
                Some(l) => l,
                None => {
                    log::warn!("fail to resolve position for {:?}", e);
                    continue;
                }
            };

            let kind = ObjKind {
                fleet: true,
                jump: j.is_some(),
                station: st.is_some(),
                asteroid: false,
                astro: false,
                astro_star: false,
                factory: false,
                shipyard: false,
            };

            let trade_orders = new_trader_orders(o);

            r.push(ObjData {
                id: e,
                coords: ls.pos,
                sector_id: ls.sector_id,
                docked: l.as_docked(),
                kind: kind,
                orbit: None,
                trade_orders: trade_orders,
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
        let astros = g.world.read_storage::<AstroBody>();
        let orbits = g.world.read_storage::<OrbitalPos>();

        let loc = (&locations).get(e)?;
        let ls = Locations::resolve_space_position_from(&locations, loc)?;
        let ab = astros.get(e);
        let orb = orbits.get(e);

        let kind = ObjKind {
            fleet: g.world.read_storage::<Fleet>().contains(e),
            jump: g.world.read_storage::<Jump>().contains(e),
            station: g.world.read_storage::<Station>().contains(e),
            asteroid: g.world.read_storage::<Extractable>().contains(e),
            astro: ab.is_some(),
            astro_star: ab.map(|ab| ab.kind == AstroBodyKind::Star).unwrap_or(false),
            factory: g.world.read_storage::<Factory>().contains(e),
            shipyard: g.world.read_storage::<Shipyard>().contains(e),
        };

        let orbit_data = orb.map(|o| {
            let parent_pos = locations.get(o.parent).and_then(|i| i.as_space()).unwrap();
            ObjOrbitData {
                radius: o.distance,
                parent_pos: parent_pos.pos,
            }
        });

        let orders = g.world.read_storage::<TradeOrders>();
        let trade_orders = orders
            .get(e)
            .map(|o| new_trader_orders(o))
            .unwrap_or_default();

        let obj = ObjData {
            id: e,
            coords: ls.pos,
            sector_id: ls.sector_id,
            docked: loc.as_docked(),
            kind: kind,
            orbit: orbit_data,
            trade_orders,
        };

        Some(obj)
    }

    pub fn get_obj_desc(&self, id: Id) -> Option<ObjDesc> {
        let g = self.game.borrow();
        let entities = g.world.entities();
        let e = decode_entity_and_get(&g, id)?;

        let docked_fleets = g
            .world
            .read_storage::<Docking>()
            .get(e)
            .map(|has_dock| {
                has_dock
                    .docked
                    .iter()
                    .map(|id| encode_entity(*id))
                    .collect()
            })
            .unwrap_or(vec![]);

        let desc = ObjDesc {
            id: id,
            label: g
                .world
                .read_storage::<Label>()
                .get(e)
                .map(|i| i.label.clone())
                .unwrap_or_else(|| "unknown".to_string()),
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
            factory: g.world.read_storage::<Factory>().get(e).cloned(),
            shipyard: g.world.read_storage::<Shipyard>().get(e).cloned(),
            docked_fleets,
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

    pub fn list_wares(&self) -> Vec<WareData> {
        let game = self.game.borrow();
        let entities = game.world.entities();
        let labels = game.world.read_storage::<Label>();
        let wares = game.world.read_storage::<Ware>();

        (&entities, &labels, &wares)
            .join()
            .map(|(e, l, _)| WareData {
                id: encode_entity(e),
                label: l.label.clone(),
            })
            .collect()
    }

    pub fn list_building_sites_prefabs(&self) -> Vec<PrefabData> {
        let game = self.game.borrow();
        let entities = game.world.entities();
        let labels = game.world.read_storage::<Label>();
        let prefabs = game.world.read_storage::<Prefab>();

        (&entities, &labels, &prefabs)
            .join()
            .filter(|(_, _, p)| p.build_site)
            .map(|(e, l, _p)| PrefabData {
                id: encode_entity(e),
                label: l.label.clone(),
            })
            .collect()
    }

    pub fn list_building_shipyard_prefabs(&self) -> Vec<PrefabData> {
        let game = self.game.borrow();
        let entities = game.world.entities();
        let labels = game.world.read_storage::<Label>();
        let prefabs = game.world.read_storage::<Prefab>();

        (&entities, &labels, &prefabs)
            .join()
            .filter(|(_, _, p)| p.shipyard)
            .map(|(e, l, _p)| PrefabData {
                id: encode_entity(e),
                label: l.label.clone(),
            })
            .collect()
    }

    pub fn new_building_plot(&self, plot_id: u64, sector_id: u64, pos_x: f32, pos_y: f32) -> u64 {
        let mut game = self.game.borrow_mut();
        let plot_id = dbg!(decode_entity_and_get(&game, plot_id).unwrap());
        let sector_id = decode_entity_and_get(&game, sector_id).unwrap();

        let prod_cost = game
            .world
            .read_storage::<Prefab>()
            .get(plot_id)
            .expect("prefab not found")
            .obj
            .production_cost
            .as_ref()
            .map(|pc| pc.cost.clone())
            .unwrap_or(vec![]);

        let new_obj = Loader::new_station_building_site(plot_id, prod_cost)
            .at_position(sector_id, P2::new(pos_x, pos_y));
        let obj_id = Loader::add_object(&mut game.world, &new_obj);
        encode_entity(obj_id)
    }

    pub fn add_shipyard_building_order(&mut self, obj_id: u64, prefab_id: u64) -> bool {
        let prefab_id = decode_entity_and_get(&self.game.borrow(), prefab_id).unwrap();
        let obj_id = decode_entity_and_get(&self.game.borrow(), obj_id).unwrap();

        let mut game = self.game.borrow_mut();

        let mut shipyards = game.world.write_storage::<Shipyard>();
        let prefabs = game.world.read_storage::<Prefab>();

        let prefab = prefabs.get(prefab_id).unwrap();
        assert!(prefab.shipyard);

        let mut shipyard = shipyards.get_mut(obj_id).unwrap();

        if !shipyard.order.is_none() {
            log::debug!("{:?} is already producing, ignoring new order", obj_id);
            return false;
        }

        shipyard.order = shipyard::ProductionOrder::Next(prefab_id);

        log::debug!("{:?} set production order to {:?}", obj_id, prefab_id);

        true
    }
}

fn new_trader_orders(o: &TradeOrders) -> Vec<ObjTradeOrder> {
    let mut trade_orders = vec![];
    trade_orders.extend(o.wares_requests().iter().map(|ware_id| {
        let ware_id = encode_entity(*ware_id);
        ObjTradeOrder {
            request: true,
            provide: false,
            ware_id,
        }
    }));
    trade_orders.extend(o.wares_provider().iter().map(|ware_id| {
        let ware_id = encode_entity(*ware_id);
        ObjTradeOrder {
            request: false,
            provide: true,
            ware_id,
        }
    }));
    trade_orders
}

include!(concat!(env!("OUT_DIR"), "/glue.rs"));

#[cfg(test)]
mod test {
    use std::num::NonZeroI32;

    use specs::world::Generation;

    use space_domain::utils::{MIN_DISTANCE, MIN_DISTANCE_SQR, V2};

    use super::*;

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
