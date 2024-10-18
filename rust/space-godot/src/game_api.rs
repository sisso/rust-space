mod label_info;
mod obj_info;
mod shipyard_info;
mod ware_amount_info;

use self::obj_info::ObjExtendedInfo;
use self::shipyard_info::ShipyardInfo;
use self::ware_amount_info::WareAmountInfo;
use crate::events::{EventsList, GameEvent};
use crate::game_api::label_info::LabelInfo;
use crate::utils;
use crate::utils::{decode_entity_and_get, encode_entity};
use bevy_ecs::prelude::*;
use commons::math::{P2, V2I};
use godot::obj::Base;
use godot::prelude::*;
use space_domain::app::App;
use space_domain::game::actions::{Action, ActionActive};
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::bevy_utils::WorldExt;
use space_domain::game::commands::Command;
use space_domain::game::events::EventKind;
use space_domain::game::extractables::Extractable;
use space_domain::game::fleets::Fleet;
use space_domain::game::game::{Game, NewGameParams};
use space_domain::game::label::Label;
use space_domain::game::loader::Loader;
use space_domain::game::locations::{LocationDocked, LocationOrbit, LocationSpace, Locations};
use space_domain::game::objects::ObjId;
use space_domain::game::order::TradeOrders;
use space_domain::game::prefab::Prefab;
use space_domain::game::save_manager::SaveManager;
use space_domain::game::sectors::{Jump, Sector};
use space_domain::game::shipyard;
use space_domain::game::shipyard::Shipyard;
use space_domain::game::station::Station;
use space_domain::game::utils::TotalTime;
use std::path::PathBuf;

pub type Id = i64;

pub const NULL_ID: Id = -1;

// #[derive(GodotClass)]
// #[class(base=RefCounter)]
// pub struct GameEvents {
//     events: Vec<GameEvent>,
//     base: Base<RefCounted>,
// }
//
// #[godot_api]
// impl GameEvents {
//     #[func]
//     fn len(&self) -> usize {
//         self.events.len()
//     }
// }
//
// #[godot_api]
// impl IRefCounted for GameEvents {
//     fn init(base: Base<RefCounted>) -> Self {
//         GameEvents {
//             events: vec![],
//             base,
//         }
//     }
// }

// // #[derive(Default, Debug, Clone)]
// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct GameEvent {
//     target_id: Id,
//     added: bool,
//     removed: bool,
// }

// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct Sectors {
//     sectors: Vec<SectorInfo>,
//     base: Base<RefCounted>,
// }
//
// #[godot_api]
// impl Sectors {
//     #[func]
//     fn len(&self) -> usize {
//         self.sectors.len()
//     }
// }
//
// #[godot_api]
// impl IRefCounted for Sectors {
//     fn init(base: Base<RefCounted>) -> Self {
//         Self {
//             sectors: vec![],
//             base,
//         }
//     }
// }

// #[derive(GodotClass)]
// #[class(base=RefCounter)]
// pub struct ObjInfo {
//     base: Base<RefCounted>,
// }
//
// #[godot_api]
// impl ObjInfo {}
//
// #[godot_api]
// impl IRefCounted for ObjInfo {
//     fn init(base: Base<RefCounted>) -> Self {
//         Self { base }
//     }
// }

// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct SectorInfo {
//     pub id: Id,
//     pub coords: Vector2i,
//     pub label: String,
// }
//
// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct FleetInfo {
//     pub id: Id,
//     pub label: String,
// }
//
// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct SpaceLocation {
//     pub sector_id: Id,
//     pub pos: Vector2,
// }
//
// #[derive(GodotClass, Debug)]
// #[class(no_init, base=RefCounter)]
// pub struct LabeledInfo {
//     pub id: i64,
//     pub label: String,
// }

struct GameRunning {
    game: Game,
    speed: f32,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    #[base]
    base: Base<Node>,
    current_game: Option<GameRunning>,
    saves: Option<SaveManager>,
}

fn resolve_log_i32(level: i32) -> log::LevelFilter {
    match level {
        0 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    }
}

impl GameRunning {
    fn get_obj_extended_info(&mut self, obj_id: ObjId) -> Option<ObjExtendedInfo> {
        let location_space = self.game.resolve_space_position(obj_id)?;
        let shipyard = self
            .get_shipyard_info(obj_id)
            .map(|value| Gd::from_object(value));
        let cargo = self.list_cargo(obj_id);
        let (requesting_wares, providing_wares) = self.list_requesting_and_providing_wares(obj_id);
        let extractable_resources = self.list_extractable_resources(obj_id);

        let mut query = self.game.world.query::<(
            Option<&Label>,
            Option<&Fleet>,
            Option<&AstroBody>,
            Option<&LocationOrbit>,
            Option<&Extractable>,
            Option<&Jump>,
            Option<&Station>,
            Option<&ActionActive>,
            Option<&Command>,
        )>();

        let (label, fleet, astro, orbit, extractable, jump, station, active_action, command) =
            query
                .get(&self.game.world, obj_id)
                .expect("fail to fetch obj data");

        let label = match label {
            Some(label) => label.label.clone(),
            None => "unknown".to_string(),
        };

        let action = match active_action {
            Some(action) => match action.get_action() {
                Action::Undock => "undock".to_string(),
                Action::Jump { .. } => "jump".to_string(),
                Action::Dock { .. } => "dock".to_string(),
                Action::MoveTo { .. } => "move to".to_string(),
                Action::MoveToTargetPos { .. } => "move to target".to_string(),
                Action::Extract { .. } => "extract".to_string(),
                Action::Orbit { .. } => "orbit".to_string(),
                Action::Deorbit => "deorbit".to_string(),
            },
            None => "none".to_string(),
        };

        let command = match command {
            Some(command) => match command {
                Command::Mine(_) => "mine".to_string(),
                Command::Trade(_) => "trade".to_string(),
            },
            None => "none".to_string(),
        };

        let is_planet = astro
            .map(|ab| match ab.kind {
                AstroBodyKind::Planet => true,
                _ => false,
            })
            .unwrap_or(false);
        let is_star = astro
            .map(|ab| match ab.kind {
                AstroBodyKind::Star => true,
                _ => false,
            })
            .unwrap_or(false);

        let kind = if is_planet {
            "planet"
        } else if is_star {
            "star"
        } else if extractable.is_some() {
            "asteroid"
        } else if fleet.is_some() {
            "fleet"
        } else if jump.is_some() {
            "jump"
        } else if station.is_some() {
            "station"
        } else {
            "unknown"
        }
        .to_string();

        let info = ObjExtendedInfo {
            id: encode_entity(obj_id),
            label,
            location_space: location_space,
            is_planet,
            is_star,
            is_asteroid: extractable.is_some(),
            is_fleet: fleet.is_some(),
            is_jump: jump.is_some(),
            is_station: station.is_some(),
            is_orbiting: orbit.is_some(),
            shipyard: shipyard,
            orbit_parent_id: orbit.map(|i| encode_entity(i.parent_id)).unwrap_or(NULL_ID),
            command,
            action,
            kind,
            cargo: cargo,
            requesting_wares: requesting_wares,
            providing_wares: providing_wares,
            resources: extractable_resources,
        };

        Some(info)
    }

    pub fn list_cargo(&mut self, obj_id: ObjId) -> Vec<WareAmountInfo> {
        let cargo = match self.game.get_cargo_of(obj_id) {
            Ok(cargo) => cargo.clone(),
            Err(_) => {
                return vec![];
            }
        };

        cargo
            .into_iter()
            .map(|wa| {
                let label = self
                    .game
                    .get_ware_label(wa.ware_id)
                    .expect("ware not found");

                WareAmountInfo {
                    id: encode_entity(wa.ware_id),
                    label: label.label.clone(),
                    amount: wa.amount as i64,
                }
            })
            .collect()
    }

    fn get_shipyard_info(&mut self, obj_id: ObjId) -> Option<ShipyardInfo> {
        let shipyard = self.game.world.get::<Shipyard>(obj_id)?;
        Some(ShipyardInfo {
            shipyard: shipyard.clone(),
        })
    }

    fn decode_entity_and_get(&mut self, id: Id) -> ObjId {
        decode_entity_and_get(&self.game, id)
    }

    // resolve the WareAmountInfo from a list of WareId
    fn resolve_ware_amount_info(&mut self, ware_ids: &[ObjId]) -> Array<Gd<LabelInfo>> {
        ware_ids
            .iter()
            .map(|ware_id| {
                let label = self.game.get_ware_label(*ware_id).expect("ware not found");
                let label_info = LabelInfo {
                    id: encode_entity(*ware_id),
                    label: label.label.clone(),
                };
                Gd::from_object(label_info)
            })
            .collect()
    }

    fn list_requesting_and_providing_wares(
        &mut self,
        obj_id: ObjId,
    ) -> (Array<Gd<LabelInfo>>, Array<Gd<LabelInfo>>) {
        let trade_orders = {
            let mut query = self.game.world.query::<&TradeOrders>();
            match query.get(&self.game.world, obj_id).ok() {
                Some(value) => value.clone(),
                None => {
                    return (Array::new(), Array::new());
                }
            }
        };

        let requesting_wares = self.resolve_ware_amount_info(&trade_orders.wares_requests());
        let providing_wares = self.resolve_ware_amount_info(&trade_orders.wares_provider());

        (requesting_wares, providing_wares)
    }

    fn list_extractable_resources(&mut self, obj_id: ObjId) -> Array<Gd<LabelInfo>> {
        let Some(extractable) = self.game.world.get::<Extractable>(obj_id) else {
            return Array::new();
        };

        self.resolve_ware_amount_info(&[extractable.ware_id])
    }
}

#[godot_api]
impl GameApi {
    #[func]
    pub fn initialize(&mut self, log_level: i32, saves_path: String) {
        // set init log
        let log_level = resolve_log_i32(log_level);
        godot_print!("using log level {:?}", log_level);
        env_logger::builder()
            .filter_level(log_level)
            // .filter(Some("space_flap"), log::LevelFilter::Warn)
            // .filter(Some("space_domain"), log::LevelFilter::Warn)
            // .filter(Some("space_domain::conf"), log::LevelFilter::Debug)
            // .filter(Some("space_domain::game::loader"), log::LevelFilter::Trace)
            .init();

        // init app
        log::info!("initializing app");

        let save_dir = PathBuf::from(saves_path);
        log::info!("save games location: {:?}", save_dir);

        if !save_dir.exists() {
            std::fs::create_dir_all(&save_dir).expect("fail to create save dir");
        }

        let saves = SaveManager::new(&save_dir).expect("fail to create save game manager");
        self.saves = Some(saves);
    }

    #[func]
    pub fn continue_or_start(&mut self) {
        if !self.continue_last_save() {
            self.start_game();
        }
    }

    #[func]
    pub fn continue_last_save(&mut self) -> bool {
        let maybe_game =
            App::continue_last(&mut self.saves.as_mut().expect("save game not initialized"));

        if let Some(game) = maybe_game {
            self.start_with_game(game);
            true
        } else {
            false
        }
    }

    fn start_with_game(&mut self, mut game: Game) {
        let (sector_id, _) = game
            .list_sectors()
            .get(0)
            .expect("game has no sector")
            .clone();

        // let wares = game.list_wares();

        self.current_game = Some(GameRunning {
            game: game,
            speed: 1.0,
        });
    }

    #[func]
    pub fn start_game(&mut self) {
        log::debug!("starting game");

        let mut params = NewGameParams::default();
        params.galaxy_size = V2I::new(2, 1);
        params.extra_fleets = 0;

        let game = Game::new(params);

        self.start_with_game(game);
    }

    #[func]
    pub fn update(&mut self, delta_time: f64) {
        log::trace!("update {:?}", delta_time);
        let has_saves = self.saves.is_some();

        let running = self.get_current();
        if running.speed > 0.0001 {
            let change = delta_time as f32 * running.speed;
            running.game.tick(change.into());
        }

        if has_saves {
            let current_tick = running.game.get_tick();
            if current_tick % 1000 == 0 {
                log::info!("autosaving...");
                let data = running.game.save_to_string();
                if let Some(err) = self
                    .saves
                    .as_mut()
                    .unwrap()
                    .write(&format!("save_{}", current_tick), data)
                    .err()
                {
                    log::warn!("fail to write save game: {}", err);
                }
            }
        }
    }

    fn get_current(&mut self) -> &mut GameRunning {
        self.current_game.as_mut().expect("game not initialized")
    }

    #[func]
    pub fn list_sectors(&mut self) -> VariantArray {
        let mut game = &mut self.get_current().game;
        game.world
            .query::<(ObjId, &Sector, &Label)>()
            .iter(&game.world)
            .map(|(id, sector, label)| {
                let d = dict! {
                   "id": encode_entity(id),
                    "label": label.label.clone(),
                    "coords": Vector2i::new(sector.coords.x, sector.coords.y)
                };
                d.to_variant()
            })
            .collect()
    }

    #[func]
    pub fn list_fleets(&mut self) -> VariantArray {
        let mut game = &mut self.get_current().game;
        game.world
            .query::<(ObjId, &Fleet, &Label)>()
            .iter(&game.world)
            .map(|(id, sector, label)| {
                let d = dict! {
                   "id": encode_entity(id),
                    "label": label.label.clone(),
                };
                d.to_variant()
            })
            .collect()
    }

    #[func]
    pub fn take_events(&mut self) -> Gd<EventsList> {
        let events = self
            .get_current()
            .game
            .take_events()
            .into_iter()
            .flat_map(|e| match e.kind {
                EventKind::Add => Some(GameEvent {
                    target_id: encode_entity(e.id),
                    added: true,
                    ..Default::default()
                }),
                _ => None,
            })
            .collect();
        EventsList::from_vec(events)
    }

    #[func]
    pub fn list_at_sector(&mut self, sector_id: Id) -> Array<i64> {
        let running = self.get_current();
        let sector_id = running.decode_entity_and_get(sector_id);
        let mut game = &mut running.game;
        game.list_at_sector(sector_id)
            .into_iter()
            .map(|obj_id| encode_entity(obj_id))
            .collect()
    }

    #[func]
    pub fn resolve_space_position(&mut self, id: Id) -> Variant {
        let running = self.get_current();
        let obj_id = running.decode_entity_and_get(id);
        let mut game = &mut running.game;
        utils::to_godot_flat_option(game.resolve_space_position(obj_id).map(|loc| {
            let d = dict! {
               "sector_id": encode_entity(loc.sector_id),
                "pos": Vector2::new(loc.pos.x, loc.pos.y)
            };
            d.to_variant()
        }))
    }

    #[func]
    pub fn describe_obj(&mut self, id: Id) -> Option<Gd<ObjExtendedInfo>> {
        let obj_id = decode_entity_and_get(&mut self.get_current().game, id);
        let running = self.get_current();
        let found = running.get_obj_extended_info(obj_id);
        found.map(|value| Gd::from_object(value))
    }

    #[func]
    pub fn list_buildings(&mut self) -> VariantArray {
        let mut game = &mut self.get_current().game;
        game.world
            .query::<(ObjId, &Prefab)>()
            .iter(&game.world)
            .filter(|(_, prefab)| prefab.build_site)
            .map(|(id, prefab)| {
                let label = prefab
                    .obj
                    .label
                    .as_ref()
                    .map(|l| l.clone())
                    .unwrap_or("unknown".to_string());

                let d = dict! {
                   "id": encode_entity(id),
                    "label": label.clone(),
                };
                d.to_variant()
            })
            .collect()
    }

    #[func]
    fn new_building_site(&mut self, sector_id: Id, pos: Vector2, prefab_id: Id) {
        let running = self.get_current();

        // resolve id and sector
        let sector_id = running.decode_entity_and_get(sector_id);
        let prefab_id = running.decode_entity_and_get(prefab_id);

        let game = &mut running.game;

        // get cost
        let prod_cost = game
            .world
            .get::<Prefab>(prefab_id)
            .expect("prefab not found")
            .obj
            .production_cost
            .as_ref()
            .map(|pc| pc.cost.clone())
            .unwrap_or(vec![]);

        // create new building plot
        let new_obj = Loader::new_station_building_site(prefab_id, prod_cost)
            .at_position(sector_id, P2::new(pos.x, pos.y));
        game.world
            .run_commands(|mut commands| Loader::add_object(&mut commands, &new_obj));
    }

    #[func]
    fn cancel_shipyard_building_order(&mut self, obj_id: Id) {
        let running = self.get_current();
        let obj_id = running.decode_entity_and_get(obj_id);
        let mut shipyard = running
            .game
            .world
            .get_mut::<Shipyard>(obj_id)
            .expect("shipyard not found");
        shipyard.set_production_order(shipyard::ProductionOrder::None);
        log::debug!("{:?} set production order to none", obj_id);
    }

    #[func]
    fn set_shipyard_building_order(&mut self, obj_id: Id, prefab_id: Id) {
        let running = self.get_current();
        let obj_id = running.decode_entity_and_get(obj_id);
        let prefab_id = running.decode_entity_and_get(prefab_id);
        let prefab = running
            .game
            .world
            .get::<Prefab>(prefab_id)
            .expect("prefab not found");
        assert!(prefab.shipyard);

        let mut shipyard = running
            .game
            .world
            .get_mut::<Shipyard>(obj_id)
            .expect("shipyard not found");
        shipyard.set_production_order(shipyard::ProductionOrder::Next(prefab_id));
        log::debug!("{:?} set production order to {:?}", obj_id, prefab_id);
    }

    #[func]
    pub fn set_speed(&mut self, speed: f32) {
        let running = self.get_current();
        running.speed = speed;
    }

    #[func]
    pub fn list_shipyards_prefabs(&mut self) -> Array<Gd<LabelInfo>> {
        let mut game = &mut self.get_current().game;
        game.world
            .query::<(ObjId, &Prefab)>()
            .iter(&game.world)
            .filter(|(_, prefab)| prefab.shipyard)
            .map(|(id, prefab)| {
                let label = prefab
                    .obj
                    .label
                    .as_ref()
                    .map(|l| l.clone())
                    .unwrap_or("unknown".to_string());

                Gd::from_object(LabelInfo {
                    id: encode_entity(id),
                    label: label,
                })
            })
            .collect()
    }

    #[func]
    pub fn get_total_time(&mut self) -> f32 {
        let mut game = &mut self.get_current().game;
        game.world
            .get_resource::<TotalTime>()
            .expect("total time not found")
            .as_f64() as f32
    }
}

#[godot_api]
impl INode for GameApi {
    fn init(base: Base<Node>) -> Self {
        GameApi {
            base: base,
            current_game: None,
            saves: None,
        }
    }
}
