use specs::prelude::*;
use std::borrow::BorrowMut;
use std::sync::Arc;

use crate::game::locations::Locations;
use crate::utils::*;

use self::new_obj::NewObj;

use self::save::{Load, Save};

use crate::game::actions::Actions;
use crate::game::astrobody::AstroBodies;
use crate::game::commands::Commands;
use crate::game::conf::Conf;

use crate::game::factory::Factory;
use crate::game::fleets::Fleet;
use crate::game::loader::Loader;
use crate::game::navigations::Navigations;
use crate::game::order::Orders;
use crate::game::sectors::Sectors;
use crate::game::shipyard::Shipyard;
use crate::game::station::Stations;
use crate::game::wares::Wares;

pub mod actions;
pub mod astrobody;
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
pub mod order;
pub mod prefab;
pub mod save;
pub mod sceneries;
pub mod scenery_random;
pub mod sectors;
pub mod ship;
pub mod shipyard;
pub mod station;
pub mod wares;

pub const FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(17);
pub const SYSTEM_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);

// TODO: add tick to game field
pub struct Game {
    pub total_time: TotalTime,
    pub world: World,
    pub dispatcher: Dispatcher<'static, 'static>,
    pub thread_pool: Arc<rayon_core::ThreadPool>,
}

pub struct GameInitContext {
    pub world: World,
    pub dispatcher: DispatcherBuilder<'static, 'static>,
}

pub trait RequireInitializer {
    fn init(context: &mut GameInitContext);
}

impl Game {
    pub fn new(conf: Conf) -> Self {
        let thread_pool: Arc<rayon_core::ThreadPool> = Arc::new(
            rayon_core::ThreadPoolBuilder::new()
                .build()
                .expect("Invalid configuration"),
        );

        let dispatcher_builder = DispatcherBuilder::new().with_pool(thread_pool.clone());

        // initialize all
        let mut init_ctx = GameInitContext {
            world: World::new(),
            dispatcher: dispatcher_builder,
        };

        // initializations
        init_ctx.world.register::<label::Label>();
        init_ctx.world.register::<code::Code>();
        init_ctx.world.register::<prefab::Prefab>();
        Sectors::init(&mut init_ctx);
        Locations::init(&mut init_ctx);
        Actions::init(&mut init_ctx);
        Commands::init(&mut init_ctx);
        Navigations::init(&mut init_ctx);
        Shipyard::init(&mut init_ctx);
        Factory::init(&mut init_ctx);
        Orders::init(&mut init_ctx);
        Stations::init(&mut init_ctx);
        Fleet::init(&mut init_ctx);
        AstroBodies::init(&mut init_ctx);
        Wares::init(&mut init_ctx);

        let mut dispatcher = init_ctx.dispatcher.build();

        let mut world = init_ctx.world;
        dispatcher.setup(&mut world);

        loader::load_prefabs(&mut world, &conf.prefabs);

        world.insert(conf);

        Game {
            total_time: TotalTime(0.0),
            world,
            dispatcher,
            thread_pool,
        }
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update time
        self.total_time = self.total_time.add(delta_time);
        self.world.insert(delta_time);
        self.world.insert(self.total_time);
        log::debug!(
            "tick delta {} total {}",
            delta_time.as_f32(),
            self.total_time.as_f64(),
        );

        // update systems
        self.dispatcher.dispatch(&mut self.world);
        // apply all lazy updates
        self.world.maintain();
        // instantiate new objects
        self.tick_new_objects_system();
    }

    pub fn save(&self, _save: &mut impl Save) {}

    pub fn load(&mut self, _load: &mut impl Load) {}

    pub fn reindex_sectors(&mut self) {
        log::info!("reindex_sectors");
        sectors::update_sectors_index(&self.world);
        locations::update_locations_index(&self.world)
    }

    pub fn tick_new_objects_system(&mut self) {
        let mut list = vec![];

        for (e, new_obj) in (
            &*self.world.entities(),
            self.world.write_storage::<NewObj>().borrow_mut().drain(),
        )
            .join()
        {
            list.push(new_obj);
            self.world.entities().delete(e).unwrap();
        }

        for obj in &list {
            Loader::add_object(&mut self.world, obj);
        }
    }
}
