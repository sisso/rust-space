#![allow(unused)]

extern crate core;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use itertools::Itertools;

use commons::math::{P2, V2I};
pub use models::*;
use space_domain::game::actions::ActionActive;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::bevy_utils::WorldExt;
use space_domain::game::conf::Conf;
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
use space_domain::game::save_manager::SaveManager;
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::wares::{Cargo, Ware};
use space_domain::game::{events, scenery_random, shipyard};
use utils::*;

pub mod models;
mod utils;

pub struct SpaceGameParams {
    pub galaxy_size: V2I,
    pub extra_fleets: usize,
    pub seed: u64,
    pub continue_latest_save: bool,
    pub save_path: Option<PathBuf>,
}

impl Default for SpaceGameParams {
    fn default() -> Self {
        Self {
            galaxy_size: V2I::new(2, 2),
            extra_fleets: 2,
            seed: 0,
            continue_latest_save: false,
            save_path: None,
        }
    }
}

/// high level API for the game
pub struct SpaceGame {
    game: Rc<RefCell<Game>>,
    saves_manager: Option<SaveManager>,
}

impl SpaceGame {
    pub fn new(params: SpaceGameParams) -> Self {
        let (saves, game) = if let Some(buffer) = &params.save_path {
            // initialize saves manager if a path is provided an try to load latest game
            let saves = SaveManager::new(buffer).expect("fail to initialize save game manager");
            let game = if params.continue_latest_save {
                if let Ok(Some(last_save)) = saves.get_last() {
                    let last_save_data = saves
                        .read(&last_save.filename)
                        .expect("fail to read save file");
                    log::info!("continue from latest save game {}", last_save.filename);
                    Game::load_from_string(last_save_data).expect("fail to read save game")
                } else {
                    log::info!("no latest save game found, starting a new game");
                    Self::start_new_game(&params)
                }
            } else {
                log::info!("starting enw game");
                Self::start_new_game(&params)
            };

            (Some(saves), game)
        } else {
            log::info!("no save game manager configured, starting new game");
            (None, Self::start_new_game(&params))
        };

        SpaceGame {
            game: Rc::new(RefCell::new(game)),
            saves_manager: saves,
        }
    }

    fn start_new_game(params: &SpaceGameParams) -> Game {
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
                size: (params.galaxy_size.x as usize, params.galaxy_size.y as usize),
                seed: params.seed,
                fleets: params.extra_fleets,
                universe_cfg: cfg.system_generator.unwrap(),
                initial_condition: scenery_random::InitialCondition::Minimal,
                params: cfg.params,
            },
        );

        game
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

        if let Some(saves) = &mut self.saves_manager {
            let current_tick = self.game.borrow().get_tick();
            if current_tick % 1000 == 0 {
                log::info!("autosaving...");
                let data = self.game.borrow_mut().save_to_string();
                saves.write(&format!("save_{}", current_tick), data);
            }
        }
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

    pub fn save(&mut self) -> Result<(), &'static str> {
        let data = self.game.borrow_mut().save_to_string();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("fail to read now timestamp");
        let save_name = format!("{}", timestamp.as_secs_f32());
        self.saves_manager
            .as_ref()
            .expect("can not save without initialize save game manager")
            .write(&save_name, data)
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
        let mut params = SpaceGameParams::default();
        params.galaxy_size = V2I::new(2, 1);
        let mut sg = SpaceGame::new(params);
    }

    #[test]
    fn test_with_default_configuration_fleets_move_around() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Trace)
            .init();

        let mut sg = SpaceGame::new(Default::default());

        // get initial fleets positions
        let mut fleets_original = sg.get_fleets();

        // wait until any fleet has position, as they started docked
        for _ in 0..100 {
            sg.update(1.0);
            fleets_original = sg.get_fleets();
            if fleets_original.iter().any(|f| f.location.is_some()) {
                break;
            }
        }

        // keep updating until fleet move a distance
        let mut changed_pos = 0;
        'out: for _ in 0..100 {
            sg.update(1.0);

            let fleets = sg.get_fleets();
            // check if they move
            for f1 in &fleets_original {
                for f2 in &fleets {
                    if f1.id == f2.id && f1.location.is_some() && f2.location.is_some() {
                        let changed = V2::distance(
                            f1.location.as_ref().unwrap().pos,
                            f2.location.as_ref().unwrap().pos,
                        ) > MIN_DISTANCE;
                        if f1.kind.fleet && changed {
                            changed_pos += 1;
                            break 'out;
                        } else if f1.kind.station && changed {
                            panic!("station should not move on {:?}", f1);
                        }
                    }
                }
            }
        }

        // get fleet positions
        assert!(changed_pos > 0);
    }

    #[test]
    fn test_proper_encode_decode_entity() {
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
        assert_eq!(9, e.generation());

        let v = proper_encode_entity(e);
        log::info!("encoded {v}");
        let (id, gen) = proper_decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.index(), id);
        assert_eq!(100, id);
        assert_eq!(e.generation(), gen);
        assert_eq!(9, gen);
    }

    #[test]
    fn test_encode_decode_entity() {
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
        assert_eq!(9, e.generation());

        let v = encode_entity(e);
        log::info!("encoded {v}");
        let (id, gen) = decode_entity(v);
        log::info!("decoded {id} {gen}");

        assert_eq!(e.index(), id);
        assert_eq!(100, id);
        assert_eq!(e.generation(), gen);
        assert_eq!(9, gen);
    }
}
