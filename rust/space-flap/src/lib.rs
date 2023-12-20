#![allow(unused)]

extern crate core;

use std::cell::RefCell;
use std::rc::Rc;

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use itertools::Itertools;

use commons::math::P2;
pub use models::*;
use space_domain::game::actions::ActionActive;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::bevy_utils::WorldExt;
use space_domain::game::dock::HasDocking;
use space_domain::game::extractables::Extractable;
use space_domain::game::factory::Factory;
use space_domain::game::fleets::Fleet;
use space_domain::game::game::Game;
use space_domain::game::label::Label;
use space_domain::game::loader;
use space_domain::game::loader::Loader;
use space_domain::game::locations::{LocationDocked, LocationOrbit, LocationSpace};
use space_domain::game::navigations::Navigation;
use space_domain::game::order::TradeOrders;
use space_domain::game::prefab::Prefab;
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::wares::{Cargo, Ware};
use space_domain::game::{events, scenery_random, shipyard};
use utils::*;

pub mod models;
mod utils;

// EOF models

pub struct SpaceGame {
    game: Rc<RefCell<Game>>,
}

impl SpaceGame {
    pub fn new(args: Vec<String>) -> Self {
        let mut size: (usize, usize) = (2, 2);
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
                        log::info!("set size to {},{}", v, v);
                        size = (v, v);
                    }
                    Err(e) => {
                        panic!("invalid argument {}={}", k, v);
                    }
                },
                "--size-xy" => {
                    let values = v.split(",").collect::<Vec<_>>();
                    if values.len() != 2 {
                        panic!("invalid argument {}={}, size-xy should contain 2 numbers separated by comma ,", k, v)
                    }

                    let x = values[0].parse::<usize>().expect("invalid argument");
                    let y = values[1].parse::<usize>().expect("invalid argument");

                    log::info!("set size to {},{}", x, y);
                    size = (x, y);
                }
                "--fleets" => match v.parse::<usize>() {
                    Ok(v) => {
                        log::info!("set fleet to {}", v);
                        fleets = v
                    }
                    Err(e) => {
                        panic!("invalid argument {}={}", k, v);
                    }
                },
                "--seed" => match v.parse::<u64>() {
                    Ok(v) => {
                        log::info!("set seed to {}", v);
                        seed = v
                    }
                    Err(e) => {
                        panic!("invalid argument {}={}", k, v);
                    }
                },
                _ => panic!("unknown argument {}={}", k, v),
            }
        }

        log::debug!("loading configuration file");
        let system_generator_conf = include_str!("../../data/game.conf");
        let cfg = space_domain::game::conf::load_str(system_generator_conf)
            .expect("fail to read config file");

        let mut game = Game::new();
        game.world.run_commands(|mut commands| {
            loader::load_prefabs(&mut commands, &cfg.prefabs);
        });
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

    fn decode_id(&self, id: Id) -> Option<Entity> {
        decode_entity_and_get(&self.game.borrow(), id)
    }

    pub fn list_at_sector(&mut self, sector_id: Id) -> Vec<Id> {
        let sector_id = self.decode_id(sector_id).expect("invalid sector id");
        self.game
            .borrow_mut()
            .list_at_sector(sector_id)
            .into_iter()
            .map(|id| encode_entity(id))
            .collect()
    }

    pub fn get_sectors(&mut self) -> Vec<SectorData> {
        let mut g = self.game.borrow_mut();
        let mut ss: SystemState<Query<(Entity, &Sector, &Label)>> = SystemState::new(&mut g.world);
        let query = ss.get(&mut g.world);

        let mut r = vec![];
        for (e, s, l) in &query {
            r.push(SectorData {
                id: encode_entity(e),
                coords: (s.coords.x, s.coords.y),
                label: l.label.clone(),
            });
        }
        r
    }

    pub fn list_jumps(&mut self) -> Vec<JumpData> {
        let mut g = self.game.borrow_mut();
        let mut ss: SystemState<Query<(Entity, &Jump)>> = SystemState::new(&mut g.world);
        let query = ss.get(&mut g.world);

        let mut r = vec![];
        for (e, _) in &query {
            r.push(JumpData {
                entity: e,
                game: self.game.clone(),
            });
        }
        r
    }

    pub fn get_jump(&mut self, id: Id) -> Option<JumpData> {
        let g = self.game.borrow();
        let e = decode_entity_and_get(&g, id)?;
        let jump = g.world.get::<Jump>(e)?;
        Some(JumpData {
            entity: e,
            game: self.game.clone(),
        })
    }

    pub fn get_label(&mut self, id: Id) -> Option<String> {
        self.game
            .borrow()
            .world
            .get::<Label>(decode_entity_and_get(&self.game.borrow(), id)?)
            .map(|l| l.label.clone())
    }

    pub fn get_fleets(&mut self) -> Vec<ObjData> {
        let mut g = self.game.borrow_mut();
        let mut ss: SystemState<
            Query<(
                Entity,
                &Fleet,
                Option<&LocationSpace>,
                Option<&LocationDocked>,
            )>,
        > = SystemState::new(&mut g.world);

        let mut ss_locations: SystemState<
            Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
        > = SystemState::new(&mut g.world);

        let query = ss.get(&g.world);
        let query_locations = ss_locations.get(&g.world);

        let mut r = vec![];
        for (id, flt, l, d) in &query {
            let kind = ObjKind {
                fleet: true,
                jump: false,
                station: false,
                asteroid: false,
                astro: false,
                astro_star: false,
                factory: false,
                shipyard: false,
            };

            r.push(ObjData {
                id,
                location: l.cloned(),
                docked: d.map(|i| i.parent_id),
                kind: kind,
                orbit: None,
                trade_orders: vec![],
            });
        }
        r
    }

    pub fn update(&mut self, delta: f32) {
        self.game.borrow_mut().tick(delta.into());
    }

    pub fn take_events(&mut self) -> Vec<EventData> {
        let events = self
            .game
            .borrow_mut()
            .world
            .resource_mut::<events::GEvents>()
            .take();
        events.into_iter().map(|i| EventData { event: i }).collect()
    }

    pub fn get_obj(&self, id: Id) -> Option<ObjData> {
        let g = self.game.borrow();
        let obj_id = decode_entity_and_get(&g, id)?;
        let entity = g.world.get_entity(obj_id)?;

        // let mut ss: SystemState<
        //     Query<(
        //         Entity,
        //         Option<&Fleet>,
        //         Option<&Jump>,
        //         Option<&Station>,
        //         Option<&Extractable>,
        //         Option<&Factory>,
        //         Option<&AstroBody>,
        //         Option<&Shipyard>,
        //     )>,
        // > = SystemState::new(&mut g.world);
        // let query = ss.get(&mut g.world);
        //
        // let locations = g.world.read_storage::<LocationSpace>();
        // let astros = g.world.read_storage::<AstroBody>();
        // let orbits = g.world.read_storage::<LocationOrbit>();
        // let loc_docked = g.world.read_storage::<LocationDocked>();

        let ls = entity.get::<LocationSpace>();
        let ab = entity.get::<AstroBody>();
        let orbit = entity.get::<LocationOrbit>();
        let docked = entity.get::<LocationDocked>();

        let kind = ObjKind {
            fleet: entity.get::<Fleet>().is_some(),
            jump: entity.get::<Jump>().is_some(),
            station: entity.get::<Station>().is_some(),
            asteroid: entity.get::<Extractable>().is_some(),
            astro: ab.is_some(),
            astro_star: ab.map(|ab| ab.kind == AstroBodyKind::Star).unwrap_or(false),
            factory: entity.get::<Factory>().is_some(),
            shipyard: entity.get::<Shipyard>().is_some(),
        };

        let orbit_data = orbit.and_then(|o| {
            let parent_pos = g.world.get::<LocationSpace>(o.parent_id)?;
            let od = ObjOrbitData {
                radius: o.distance,
                parent_pos: parent_pos.pos,
            };
            Some(od)
        });

        let trade_orders = entity
            .get::<TradeOrders>()
            .map(|o| new_trader_orders(o))
            .unwrap_or_default();

        let obj = ObjData {
            id: obj_id,
            location: ls.cloned(),
            docked: docked.map(|d| d.parent_id),
            kind: kind,
            orbit: orbit_data,
            trade_orders,
        };

        Some(obj)
    }

    pub fn get_obj_desc(&self, id: Id) -> Option<ObjDesc> {
        let g = self.game.borrow();
        let obj_id = decode_entity_and_get(&g, id)?;
        let entity = g.world.get_entity(obj_id)?;

        let docked_fleets = entity
            .get::<HasDocking>()
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
            label: entity
                .get::<Label>()
                .map(|i| i.label.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            extractable: entity.get::<Extractable>().map(|ext| ext.ware_id),
            action: entity
                .get::<ActionActive>()
                .map(|action| action.get_action().clone()),
            nav_move_to: entity.get::<Navigation>().cloned(),
            cargo: entity.get::<Cargo>().cloned(),
            factory: entity.get::<Factory>().cloned(),
            shipyard: entity.get::<Shipyard>().cloned(),
            docked_fleets,
        };

        Some(desc)
    }

    pub fn get_obj_coords(&self, id: Id) -> Option<ObjCoords> {
        let game = self.game.borrow();
        let e = decode_entity_and_get(&game, id)?;
        let location = game.world.get::<LocationSpace>(e);
        let loc_docked = game.world.get::<LocationDocked>(e);
        Some(ObjCoords {
            location: location.cloned(),
            is_docked: loc_docked.is_some(),
        })
    }

    pub fn list_wares(&mut self) -> Vec<WareData> {
        let mut g = self.game.borrow_mut();
        let mut ss: SystemState<Query<(Entity, &Label, &Ware)>> = SystemState::new(&mut g.world);
        let query = ss.get(&mut g.world);

        query
            .iter()
            .map(|(id, label, ware)| WareData {
                id: encode_entity(id),
                label: label.label.clone(),
            })
            .collect()
    }

    pub fn list_building_prefabs(&mut self) -> Vec<PrefabData> {
        let mut g = self.game.borrow_mut();
        let mut ss: SystemState<Query<(Entity, &Label, &Prefab)>> = SystemState::new(&mut g.world);
        let query = ss.get(&mut g.world);

        query
            .iter()
            .map(|(id, label, prefab)| PrefabData {
                id: encode_entity(id),
                label: label.label.clone(),

                shipyard: prefab.shipyard,
                building_site: prefab.build_site,
            })
            .collect()
    }

    pub fn list_building_sites_prefabs(&mut self) -> Vec<PrefabData> {
        self.list_building_prefabs()
            .into_iter()
            .filter(|i| i.building_site)
            .collect()
    }

    pub fn list_building_shipyard_prefabs(&mut self) -> Vec<PrefabData> {
        self.list_building_prefabs()
            .into_iter()
            .filter(|i| i.shipyard)
            .collect()
    }

    pub fn new_building_plot(
        &mut self,
        plot_id: u64,
        sector_id: u64,
        pos_x: f32,
        pos_y: f32,
    ) -> u64 {
        let mut game = self.game.borrow_mut();
        let plot_id = dbg!(decode_entity_and_get(&game, plot_id).unwrap());
        let sector_id = decode_entity_and_get(&game, sector_id).unwrap();

        let prod_cost = game
            .world
            .get::<Prefab>(plot_id)
            .expect("prefab not found")
            .obj
            .production_cost
            .as_ref()
            .map(|pc| pc.cost.clone())
            .unwrap_or(vec![]);

        let new_obj = Loader::new_station_building_site(plot_id, prod_cost)
            .at_position(sector_id, P2::new(pos_x, pos_y));
        let obj_id = game
            .world
            .run_commands(|mut commands| Loader::add_object(&mut commands, &new_obj));
        encode_entity(obj_id)
    }

    pub fn set_shipyard_building_order(&mut self, obj_id: u64, prefab_id: u64) {
        let prefab_id = decode_entity_and_get(&self.game.borrow(), prefab_id).unwrap();
        let obj_id = decode_entity_and_get(&self.game.borrow(), obj_id).unwrap();

        let mut game = self.game.borrow_mut();
        let prefab = game
            .world
            .get::<Prefab>(prefab_id)
            .expect("prefab not found");
        assert!(prefab.shipyard);

        let mut shipyard = game
            .world
            .get_mut::<Shipyard>(obj_id)
            .expect("shipyard not found");
        shipyard.set_production_order(shipyard::ProductionOrder::Next(prefab_id));
        log::debug!("{:?} set production order to {:?}", obj_id, prefab_id);
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

#[cfg(test)]
mod test {
    use super::*;
    use space_domain::game::utils::{MIN_DISTANCE, V2};
    #[test]
    fn test_arguments() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .try_init();

        let mut sg = SpaceGame::new(vec!["--size-xy".into(), "2,1".into()]);
    }

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
                    let changed = V2::distance(f.location.unwrap().pos, f2.location.unwrap().pos)
                        > MIN_DISTANCE;
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
            w.spawn_empty().id();
        }

        for _ in 0..9 {
            let e = w.spawn_empty().id();
            w.despawn(e);
        }

        let e = w.spawn_empty().id();
        assert_eq!(100, e.index());
        assert_eq!(10, e.generation());

        let v = proper_encode_entity(e);
        log::info!("encoded {v}");
        let (id, gen) = proper_decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.index(), id);
        assert_eq!(100, id);
        assert_eq!(e.generation(), gen);
        assert_eq!(10, gen);
    }

    #[test]
    fn test_encode_decode_entity() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .try_init();

        let mut w = World::new();
        for _ in 0..100 {
            w.spawn_empty().id();
        }

        for _ in 0..9 {
            let e = w.spawn_empty().id();
            w.despawn(e);
        }

        let e = w.spawn_empty().id();
        assert_eq!(100, e.index());
        assert_eq!(10, e.generation());

        let v = encode_entity(e);
        log::info!("encoded {v}");
        let (id, gen) = decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.index(), id);
        assert_eq!(100, id);
        assert_eq!(e.generation(), gen);
        assert_eq!(10, gen);
    }
}
