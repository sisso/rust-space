use std::borrow::BorrowMut;
use std::sync::Arc;

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::InternedScheduleLabel;
use bevy_ecs::system::RunSystemOnce;

use crate::game::actions::Actions;
use crate::game::astrobody::AstroBodies;
use crate::game::commands::FleetCommands;
use crate::game::events::GEvent;
use crate::game::factory::Factory;
use crate::game::fleets::Fleet;
use crate::game::loader::Loader;
use crate::game::locations::Locations;
use crate::game::navigations::Navigations;
use crate::game::orbit::Orbits;
use crate::game::order::TradeOrders;
use crate::game::sectors::Sectors;
use crate::game::shipyard::Shipyard;
use crate::game::station::Stations;
use crate::game::utils::{DeltaTime, TotalTime};
use crate::game::wares::Wares;

use self::new_obj::NewObj;
use self::save::{Load, Save};

pub mod actions;
pub mod astrobody;
mod bevy_utils;
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
pub mod save;
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

pub struct Game {
    pub world: World,
    pub scheduler: Schedule,
}

// TODO: merge into game
pub struct GameInitContext {
    pub world: World,
    pub scheduler: Schedule,
}

pub trait RequireInitializer {
    fn init(context: &mut GameInitContext);
}

impl Game {
    pub fn new() -> Self {
        let scheduler = Schedule::default();

        // initialize all
        let mut init_ctx = GameInitContext {
            world: World::new(),
            scheduler,
        };

        init_ctx.world.insert_resource(TotalTime(0.0));
        init_ctx.world.init_resource::<Events<GEvent>>();

        // initializations
        Sectors::init(&mut init_ctx);
        Locations::init(&mut init_ctx);
        Actions::init(&mut init_ctx);
        FleetCommands::init(&mut init_ctx);
        Navigations::init(&mut init_ctx);
        Shipyard::init(&mut init_ctx);
        Factory::init(&mut init_ctx);
        TradeOrders::init(&mut init_ctx);
        Stations::init(&mut init_ctx);
        Fleet::init(&mut init_ctx);
        AstroBodies::init(&mut init_ctx);
        Wares::init(&mut init_ctx);
        Orbits::init(&mut init_ctx);

        // init_ctx
        //     .scheduler
        //     .add_systems(building_site::BuildingSystem);

        Game {
            world: init_ctx.world,
            scheduler: init_ctx.scheduler,
        }
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

    pub fn save(&self, _save: &mut impl Save) {}

    pub fn load(&mut self, _load: &mut impl Load) {}

    pub fn reindex_sectors(&mut self) {
        log::trace!("reindex_sectors");
        sectors::update_sectors_index(&self.world);
        locations::update_locations_index(&self.world)
    }

    fn tick_new_objects_system(mut commands: Commands, query: Query<(Entity, &NewObj)>) {
        for (obj_id, new_obj) in query {
            Loader::add_object(&mut commands, new_obj);
            commands.entity(obj_id).despawn();
        }
    }

    pub fn debug_dump(&self) {
        let labels = self.world.read_storage::<label::Label>();
        let entities = self.world.entities();

        for (e, l) in (&entities, labels.maybe()).join() {
            log::debug!(
                "{} {}",
                e.id(),
                l.map(|l| l.label.as_str()).unwrap_or("unknown")
            )
        }
    }
}
