use specs::prelude::*;
use crate::specs_extras::*;
use std::collections::HashMap;

use crate::game::extractables::Extractable;
use crate::game::locations::{LocationDock, LocationSpace, Moveable, Locations};
use crate::utils::*;

use self::events::{EventKind, Events, ObjEvent};
use self::extractables::Extractables;
use self::new_obj::NewObj;
use self::objects::*;
use self::save::{CanLoad, CanSave, Load, Save};
use self::sectors::*;
use self::wares::*;
use crate::game::navigations::Navigations;
use crate::game::commands::{Commands, Command, CommandMine};
use crate::game::actions::Actions;
use std::borrow::BorrowMut;

pub mod sectors;
pub mod objects;
pub mod wares;
pub mod actions;
pub mod commands;
pub mod navigations;
pub mod locations;
pub mod extractables;
pub mod save;
pub mod new_obj;
pub mod jsons;
pub mod ship;
//pub mod factory;
pub mod events;
//pub mod ai_high;

// TODO: add tick to game field
// TODO: remove most of fields?
pub struct Game<'a, 'b> {
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'b>,
//    pub commands: Commands,
//    pub actions: Actions,
//    pub sectors: Sectors,
//    pub objects: Objects,
//    pub locations: Locations,
//    pub extractables: Extractables,
//    pub cargos: Cargos,
//    pub events: Events,
//    pub navigations: Navigations,
}

impl <'a, 'b> Game <'a, 'b>{
    pub fn new() -> Self {
        let mut world = World::new();
        let mut dispatcher_builder = DispatcherBuilder::new();

        Actions::init_world(&mut world, &mut dispatcher_builder);
        Commands::init_world(&mut world, &mut dispatcher_builder);
        Navigations::init_world(&mut world, &mut dispatcher_builder);
        Locations::init_world(&mut world, &mut dispatcher_builder);
        Extractables::init_world(&mut world, &mut dispatcher_builder);
        Objects::init_world(&mut world, &mut dispatcher_builder);
        Cargos::init_world(&mut world, &mut dispatcher_builder);
        Navigations::init_world(&mut world, &mut dispatcher_builder);

        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);

        Game {
            world,
            dispatcher,
//            commands: Commands::new(),
//            actions: Actions::new(),
//            sectors: Sectors::new(),
//            objects: Objects::new(),
//            locations: Locations::new(),
//            extractables: Extractables::new(),
//            cargos: Cargos::new(),
//            events: Events::new(),
//            navigations: Navigations::new(),
        }
    }

    pub fn add_object(&mut self, new_obj: NewObj) -> ObjId {
        let mut builder = self.world.create_entity();

        if new_obj.has_dock {
            builder.set(HasDock);
        }
//        self.locations.init(&id);
//
//        if new_obj.ai {
//            self.commands.init(id);
//            self.actions.init(id);
//        }

        for location_space in new_obj.location_space {
            builder.set(location_space);
        }

        for location_dock in new_obj.location_dock {
            builder.set(location_dock);
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

//        self.events.add_obj_event(ObjEvent::new(id, EventKind::Add));


        builder.build()
    }

    pub fn set_command(&mut self, entity: Entity, command: Command) {
        let mut storage = self.world.write_storage::<Command>();
        storage.borrow_mut().insert(entity, command);

        let mut storage = self.world.write_storage::<CommandMine>();
        storage.borrow_mut().insert(entity, CommandMine);
    }

    pub fn tick(&mut self, total_time: TotalTime, delta_time: DeltaTime) {
        info!("game", &format!("tick delta {} total {}", delta_time.0, total_time.0));
        self.world.insert(delta_time);
        self.world.insert(total_time);
        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
    }

    pub fn save(&self, save: &mut impl Save) {
    }

    pub fn load(&mut self, load: &mut impl Load) {
    }

    // TODO: make sense?
    pub fn set_sectors(&mut self, sectors: Sectors) {
        self.world.insert(sectors);
    }
}
