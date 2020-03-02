use specs::prelude::*;
use std::collections::HashMap;
use std::borrow::BorrowMut;

use crate::game::extractables::Extractable;
use crate::game::locations::{Location, Locations, Moveable};
use crate::utils::*;

use self::extractables::Extractables;
use self::new_obj::NewObj;
use self::objects::*;
use self::save::{CanLoad, CanSave, Load, Save};
use self::sectors::*;
use self::wares::*;
use crate::game::actions::Actions;
use crate::game::commands::{CommandMine, Commands};
use crate::game::navigations::Navigations;
use crate::game::events::{Events, Event, EventKind};
use crate::ffi::{FFIApi, FFI};
use crate::game::shipyard::Shipyard;
use crate::game::dock::HasDock;
use crate::game::loader::Loader;

pub mod actions;
pub mod commands;
pub mod extractables;
pub mod jsons;
pub mod locations;
pub mod navigations;
pub mod new_obj;
pub mod objects;
pub mod save;
pub mod sectors;
pub mod ship;
pub mod wares;
pub mod events;
pub mod loader;
pub mod shipyard;
pub mod dock;
pub mod station;
pub mod factory;

// TODO: add tick to game field
pub struct Game<'a, 'b> {
    pub total_time: TotalTime,
    pub world: World,
    /// Normal dispatcher
    pub dispatcher: Dispatcher<'a, 'b>,
    /// Dispatchers that execute after normal execution and after all lazy update get applied
    /// This dispatcher is where all events should be processed
    pub late_dispatcher: Dispatcher<'a, 'b>,
    pub cleanup_dispatcher: Dispatcher<'a, 'b>,
}

pub struct GameInitContext<'a, 'b, 'c> {
    pub world: &'c mut World,
    pub dispatcher: DispatcherBuilder<'a, 'b>,
    pub late_dispatcher: DispatcherBuilder<'a, 'b>,
    pub cleanup_dispatcher: DispatcherBuilder<'a, 'b>,
}

pub trait RequireInitializer {
    fn init(context: &mut GameInitContext);
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new() -> Self {
        let mut world = World::new();

        // initialize all
        let mut init_context = GameInitContext {
            world: &mut world,
            dispatcher: Default::default(),
            late_dispatcher: Default::default(),
            cleanup_dispatcher: Default::default()
        };

        // initializations
        Sectors::init(&mut init_context);
        Locations::init(&mut init_context);
        Actions::init(&mut init_context);
        Commands::init(&mut init_context);
        Navigations::init(&mut init_context);
        Shipyard::init(&mut init_context);
        FFI::init(&mut init_context);
        Events::init(&mut init_context);

        let mut dispatcher = init_context.dispatcher.build();
        let mut late_dispatcher = init_context.late_dispatcher.build();
        let mut cleanup_dispatcher = init_context.cleanup_dispatcher.build();

        dispatcher.setup(&mut world);
        late_dispatcher.setup(&mut world);
        cleanup_dispatcher.setup(&mut world);

        Game {
            total_time: TotalTime(0.0),
            world,
            dispatcher,
            late_dispatcher,
            cleanup_dispatcher,
        }
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update time
        self.total_time = self.total_time.add(delta_time);
        self.world.insert(delta_time);
        self.world.insert(self.total_time);
        info!("tick delta {} total {}", delta_time.as_f32(), self.total_time.as_f64());

        // update systems
        self.dispatcher.dispatch(&mut self.world);
        // instantiate new objects
        self.tick_new_objects_system();
        // apply all lazy updates
        self.world.maintain();
        // run later dispatcher (this dispatcher is not calling maintain
        self.late_dispatcher.dispatch(&mut self.world);
        // run later dispatcher
        self.cleanup_dispatcher.dispatch(&mut self.world);
        // apply all lazy updates from later dispatcher
        self.world.maintain();
    }

    pub fn save(&self, save: &mut impl Save) {}

    pub fn load(&mut self, load: &mut impl Load) {}

    pub fn reindex_sectors(&mut self) {
        info!("reindex_sectors");
        SectorsIndex::update_index_from_world(&mut self.world);
    }

    pub fn tick_new_objects_system(&mut self) {
        let mut list = vec![];

        for (e, new_obj) in (&*self.world.entities(), self.world.write_storage::<NewObj>().borrow_mut().drain()).join() {
            list.push(new_obj);
            self.world.entities().delete(e);
        }

        for obj in list {
            Loader::add_object(&mut self.world, obj);
        }
    }
}

