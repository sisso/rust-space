use crate::game::bevy_utils::WorldExt;
use crate::game::events::{GEvent, GEvents};
use crate::game::label::Label;
use crate::game::loader::Loader;
use crate::game::locations::{
    update_entity_per_sector_index, EntityPerSectorIndex, LocationSpace, Locations,
};
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use crate::game::sectors::{Sector, SectorId};
use crate::game::utils::{DeltaTime, Tick, TotalTime};
use crate::game::wares::WareAmount;
use crate::game::{
    actions, building_site, commands, conf, factory, label, loader, locations, navigations, orbit,
    save, scenery_random, sectors, shipyard, wares,
};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryIter;
use bevy_ecs::system::{RunSystemOnce, SystemState};
use commons::math::V2I;
use std::path::PathBuf;

pub const FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(17);
pub const SYSTEM_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);

pub struct NewGameParams {
    pub galaxy_size: V2I,
    pub extra_fleets: usize,
    pub seed: u64,
}

impl Default for NewGameParams {
    fn default() -> Self {
        Self {
            galaxy_size: V2I::new(2, 2),
            extra_fleets: 2,
            seed: 0,
        }
    }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemSeq {
    Before,
    Ai,
    Changes,
    After,
}

pub struct Game {
    pub world: World,
    pub scheduler: Schedule,
}

impl Game {
    pub fn empty() -> Self {
        log::trace!("new game");
        let mut game = Game {
            world: World::new(),
            scheduler: Schedule::default(),
        };

        // configure
        game.scheduler.configure_sets(
            (
                SystemSeq::Before,
                SystemSeq::Ai,
                SystemSeq::Changes,
                SystemSeq::After,
            )
                .chain(),
        );

        // add resources
        game.world.insert_resource(TotalTime(0.0));
        game.world.insert_resource(GEvents::default());
        game.world.init_resource::<Events<GEvent>>();
        game.world.insert_resource(EntityPerSectorIndex::new());
        game.world.insert_resource(Tick::default());

        // ai
        game.scheduler
            .add_systems(commands::command_mine_system::system_command_mine.in_set(SystemSeq::Ai));
        game.scheduler.add_systems(
            commands::command_trader_system::system_command_trade.in_set(SystemSeq::Ai),
        );
        // changes
        game.scheduler
            .add_systems(building_site::system_building_site.in_set(SystemSeq::Changes));
        game.scheduler.add_systems(
            navigations::navigation_system::system_navigation.in_set(SystemSeq::Changes),
        );
        game.scheduler.add_systems(
            navigations::navigation_request_handler_system::system_navigation_request
                .in_set(SystemSeq::Changes),
        );
        game.scheduler
            .add_systems(factory::system_factory.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(shipyard::system_shipyard.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(orbit::system_compute_orbits.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(actions::action_dock_system::system_dock.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(actions::action_extract_system::system_extract.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(actions::action_jump_system::system_jump.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(actions::action_move_to_system::system_move.in_set(SystemSeq::Changes));
        game.scheduler.add_systems(
            actions::action_request_handler_system::system_action_request
                .in_set(SystemSeq::Changes),
        );
        game.scheduler
            .add_systems(actions::action_undock_system::system_undock.in_set(SystemSeq::Changes));
        game.scheduler
            .add_systems(actions::actions_system::system_actions.in_set(SystemSeq::Changes));
        // after
        game.scheduler
            .add_systems(locations::update_entity_per_sector_index.in_set(SystemSeq::After));
        game.scheduler
            .add_systems(wares::system_cargo_distribution.in_set(SystemSeq::After));
        game.scheduler
            .add_systems(sectors::system_update_sectors_index.in_set(SystemSeq::After));
        game.scheduler
            .add_systems(system_tick_new_objects.in_set(SystemSeq::After));

        game
    }

    pub fn new(params: NewGameParams) -> Game {
        log::info!("starting a new game");

        let system_generator_conf = include_str!("../../../data/game.conf");
        let cfg = conf::load_str(system_generator_conf).expect("fail to read config file");

        let mut game = Game::empty();

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

    pub fn load_from_string(data: String) -> Result<Game, &'static str> {
        log::trace!("load game");
        let mut game = Game::empty();
        save::load_world(&mut game.world, data);
        game.reindex_sectors();
        Ok(game)
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update tick
        self.world.get_resource_mut::<Tick>().unwrap().increment();

        // update time
        self.world.insert_resource(delta_time);
        {
            let total_time = self.world.get_resource::<TotalTime>().unwrap();
            let total_time = total_time.add(delta_time);
            log::trace!(
                "tick delta {} total {}",
                delta_time.as_f32(),
                total_time.as_f64(),
            );
            self.world.insert_resource(total_time);
        }

        // update systems
        self.scheduler.run(&mut self.world);
    }

    pub fn get_tick(&self) -> u64 {
        self.world.get_resource::<Tick>().unwrap().value()
    }

    pub fn reindex_sectors(&mut self) {
        log::trace!("reindex_sectors");
        self.world
            .run_system_once(sectors::system_update_sectors_index);
        self.world.run_system_once(update_entity_per_sector_index);
    }

    pub fn debug_dump(&mut self) {
        fn dump(query: Query<(Entity, Option<&label::Label>)>) {
            for (e, l) in &query {
                log::debug!(
                    "{:?} {}",
                    e,
                    l.map(|l| l.label.as_str()).unwrap_or("unknown")
                )
            }
        }

        self.world.run_system_once(dump);
    }

    pub fn get_cargo_of(&mut self, obj_id: ObjId) -> Result<&Vec<WareAmount>, &'static str> {
        let mut query = self.world.query::<&wares::Cargo>();
        match query.get(&self.world, obj_id) {
            Ok(cargo) => Ok(cargo.get_wares()),
            Err(_) => Err("not found"),
        }
    }

    pub fn get_ware_label(&mut self, ware_id: ObjId) -> Result<&Label, &'static str> {
        let mut query = self.world.query::<&label::Label>();
        match query.get(&self.world, ware_id) {
            Ok(label) => Ok(label),
            Err(_) => Err("not found"),
        }
    }
}

impl Game {
    pub fn list_sectors(&mut self) -> Vec<(Entity, &Sector)> {
        self.world
            .query::<(Entity, &Sector)>()
            .iter(&self.world)
            .collect()
    }

    pub fn list_at_sector(&mut self, sector_id: SectorId) -> Vec<Entity> {
        let mut ss: SystemState<Query<(Entity, &LocationSpace)>> =
            SystemState::new(&mut self.world);
        let query = ss.get(&self.world);
        query
            .iter()
            .filter(|(_, l)| l.sector_id == sector_id)
            .map(|(id, _)| id)
            .collect()
    }

    pub fn resolve_space_position(&mut self, obj_id: ObjId) -> Option<LocationSpace> {
        self.world
            .run_system_once_with(obj_id, Locations::resolve_space_position_system)
    }

    pub fn save_to_string(&mut self) -> String {
        save::save_world(&mut self.world)
    }

    pub fn take_events(&mut self) -> Vec<GEvent> {
        self.world.resource_mut::<GEvents>().take()
    }
}

fn system_tick_new_objects(mut commands: Commands, query: Query<(Entity, &NewObj)>) {
    for (obj_id, new_obj) in &query {
        log::info!(
            "using deprecated new object creation for {:?} {:?}",
            obj_id,
            new_obj
        );
        Loader::add_object(&mut commands, new_obj);
        commands.entity(obj_id).despawn();
    }
}
