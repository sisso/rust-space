use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

use crate::game::events::{GEvent, GEvents};
use crate::game::loader::Loader;
use crate::game::locations::EntityPerSectorIndex;
use crate::game::utils::{DeltaTime, TotalTime};

use self::new_obj::NewObj;

pub mod actions;
pub mod astrobody;
pub mod bevy_utils;
pub mod building_site;
pub mod code;
pub mod commands;
pub mod conf;
pub mod dock;
pub mod events;
pub mod extractables;
pub mod factory;
pub mod fleets;
pub mod jsons;
pub mod label;
pub mod loader;
pub mod locations;
pub mod navigations;
pub mod new_obj;
pub mod objects;
pub mod orbit;
pub mod order;
pub mod prefab;
pub mod production_cost;
pub mod sceneries;
pub mod scenery_random;
pub mod sectors;
pub mod ship;
pub mod shipyard;
pub mod station;
pub mod utils;
pub mod wares;
pub mod work;

pub const FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(17);
pub const SYSTEM_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemSeq {
    Before,
    AI,
    Changes,
    After,
}

pub struct Game {
    pub world: World,
    pub scheduler: Schedule,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            world: World::new(),
            scheduler: Schedule::default(),
        };

        // configure
        game.scheduler.configure_sets(
            (
                SystemSeq::Before,
                SystemSeq::AI,
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

        // ai
        game.scheduler.add_systems(
            commands::command_mine_system::system_command_mine.in_set(SystemSeq::Changes),
        );
        game.scheduler.add_systems(
            commands::command_trader_system::system_command_trade.in_set(SystemSeq::Changes),
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
            .add_systems(wares::system_cargo_distribution.in_set(SystemSeq::Changes));
        // after
        game.scheduler
            .add_systems(locations::update_entity_per_sector_index.in_set(SystemSeq::After));
        game.scheduler
            .add_systems(wares::system_cargo_distribution.in_set(SystemSeq::After));
        game.scheduler
            .add_systems(sectors::system_update_sectors_index.in_set(SystemSeq::After));

        game
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update time
        self.world.insert_resource(delta_time);
        let total_time = self.world.get_resource_mut::<TotalTime>().unwrap();
        total_time.add(delta_time);
        log::trace!(
            "tick delta {} total {}",
            delta_time.as_f32(),
            total_time.as_f64(),
        );
        drop(total_time);

        // update systems
        // self.dispatcher.dispatch(&mut self.world);
        // apply all lazy updates
        // self.world.maintain();
        // instantiate new objects
        self.world.run_system_once(Self::tick_new_objects_system);
    }

    pub fn reindex_sectors(&mut self) {
        log::trace!("reindex_sectors");
        self.world
            .run_system_once(sectors::system_update_sectors_index);
        locations::force_update_locations_index(&mut self.world)
    }

    fn tick_new_objects_system(mut commands: Commands, query: Query<(Entity, &NewObj)>) {
        for (obj_id, new_obj) in &query {
            Loader::add_object(&mut commands, new_obj);
            commands.entity(obj_id).despawn();
        }
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
}
