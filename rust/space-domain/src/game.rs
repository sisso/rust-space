use specs::prelude::*;
use std::borrow::BorrowMut;

use crate::game::locations::Locations;
use crate::utils::*;

use self::new_obj::NewObj;

use self::save::{Load, Save};
use self::sectors::*;

use crate::ffi::FFI;
use crate::game::actions::Actions;
use crate::game::commands::Commands;

use crate::game::factory::Factory;
use crate::game::loader::Loader;
use crate::game::navigations::Navigations;
use crate::game::order::Orders;
use crate::game::shipyard::Shipyard;
use crate::game::station::Stations;

pub mod actions;
pub mod commands;
pub mod dock;
pub mod events;
pub mod extractables;
pub mod factory;
pub mod jsons;
pub mod label;
pub mod loader;
pub mod locations;
pub mod navigations;
pub mod new_obj;
pub mod objects;
pub mod order;
pub mod save;
pub mod sectors;
pub mod ship;
pub mod shipyard;
pub mod station;
pub mod wares;

// TODO: add tick to game field
pub struct Game {
    pub total_time: TotalTime,
    pub world: World,
    pub dispatcher: Dispatcher<'static, 'static>,
}

pub struct GameInitContext {
    pub world: World,
    pub dispatcher: DispatcherBuilder<'static, 'static>,
    pub late_dispatcher: DispatcherBuilder<'static, 'static>,
}

pub trait RequireInitializer {
    fn init(context: &mut GameInitContext);
}

impl Game {
    pub fn new() -> Self {
        // initialize all
        let mut ictx = GameInitContext {
            world: World::new(),
            dispatcher: Default::default(),
            late_dispatcher: Default::default(),
        };

        // initializations
        ictx.world.register::<label::Label>();
        Sectors::init(&mut ictx);
        Locations::init(&mut ictx);
        Actions::init(&mut ictx);
        Commands::init(&mut ictx);
        Navigations::init(&mut ictx);
        Shipyard::init(&mut ictx);
        FFI::init(&mut ictx);
        Factory::init(&mut ictx);
        Orders::init(&mut ictx);
        Stations::init(&mut ictx);

        let mut dispatcher = ictx.dispatcher.build();

        let mut world = ictx.world;
        dispatcher.setup(&mut world);

        Game {
            total_time: TotalTime(0.0),
            world,
            dispatcher,
        }
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update time
        self.total_time = self.total_time.add(delta_time);
        self.world.insert(delta_time);
        self.world.insert(self.total_time);
        info!(
            "tick delta {} total {}",
            delta_time.as_f32(),
            self.total_time.as_f64()
        );

        // update systems
        self.dispatcher.dispatch(&mut self.world);
        // instantiate new objects
        self.tick_new_objects_system();
        // apply all lazy updates
        self.world.maintain();
        // apply all lazy updates from later dispatcher
        self.world.maintain();
    }

    pub fn save(&self, _save: &mut impl Save) {}

    pub fn load(&mut self, _load: &mut impl Load) {}

    pub fn reindex_sectors(&mut self) {
        info!("reindex_sectors");
        SectorsIndex::update_index_from_world(&mut self.world);
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

        for obj in list {
            Loader::add_object(&mut self.world, obj);
        }
    }
}
