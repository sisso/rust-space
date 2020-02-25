use crate::specs_extras::*;
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
use crate::game::station::Station;
use crate::game::events::{Events, Event, EventKind};
use crate::ffi::FFIApi;
use crate::game::factory::Factory;

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
//pub mod factory;
pub mod events;
//pub mod ai_high;
pub mod station;
pub mod loader;
pub mod factory;

// TODO: add tick to game field
pub struct Game<'a, 'b> {
    pub total_time: TotalTime,
    pub world: World,
    /// Normal dispatcher
    pub dispatcher: Dispatcher<'a, 'b>,
    /// Dispatchers that execute after normal execution and all lazy update get applied
    pub late_dispatcher: Dispatcher<'a, 'b>,
    pub cleanup_dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> Game<'a, 'b> {
    pub fn new() -> Self {
        let mut world = World::new();

        // initializations
        Sectors::init_world(&mut world);

        // normal dispatcher
        let mut dispatcher_builder = DispatcherBuilder::new();
        Locations::init_world(&mut world, &mut dispatcher_builder);
        Actions::init_world(&mut world, &mut dispatcher_builder);
        Commands::init_world(&mut world, &mut dispatcher_builder);
        Navigations::init_world(&mut world, &mut dispatcher_builder);
        Extractables::init_world(&mut world, &mut dispatcher_builder);
        Objects::init_world(&mut world, &mut dispatcher_builder);
        Cargos::init_world(&mut world, &mut dispatcher_builder);
        Factory::init_world(&mut world, &mut dispatcher_builder);

        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);

        // later dispatcher
        let mut dispatcher_builder = DispatcherBuilder::new();
        FFIApi::init_world(&mut world, &mut dispatcher_builder);

        let mut late_dispatcher = dispatcher_builder.build();
        late_dispatcher.setup(&mut world);

        // clean up dispatcher
        let mut dispatcher_builder = DispatcherBuilder::new();
        Events::init_world_cleanup(&mut world, &mut dispatcher_builder);

        let mut cleanup_dispatcher = dispatcher_builder.build();
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
            self.add_object(obj);
        }
    }

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        let mut builder = self.world.create_entity();

        if new_obj.can_dock && new_obj.speed.is_none() {
            panic!(format!(
                "fatal {:?}: entity that can dock should be moveable",
                new_obj
            ));
        }

        if new_obj.has_dock {
            builder.set(HasDock);
        }

        for location in &new_obj.location {
            builder.set(location.clone());
        }

        for speed in new_obj.speed {
            builder.set(Moveable { speed });
        }

        new_obj.extractable.iter().for_each(|i| {
            builder.set(i.clone());
        });

        if new_obj.cargo_size > 0.0 {
            let cargo = Cargo::new(new_obj.cargo_size);
            builder.set(cargo);
        }

        if new_obj.station {
            builder.set(Station {});
        }

        if new_obj.sector {
            builder.set(Sector {});
        }

        if let Some(target_id) = new_obj.jump_to {
            builder.set(Jump { target_id });
        }

        if new_obj.command_mine {
            builder.set(CommandMine::new());
        }

        if new_obj.factory {
            builder.set(Factory::new());
        }

        let entity = builder.build();

        info!("add_object {:?} from {:?}", entity, new_obj);

        self.world.create_entity()
            .with(Event::new(entity, EventKind::Add))
            .build();

        entity
    }
}
