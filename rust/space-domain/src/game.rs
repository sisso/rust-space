use crate::specs_extras::*;
use specs::prelude::*;
use std::collections::HashMap;

use crate::game::extractables::Extractable;
use crate::game::locations::{Location, Locations, Moveable};
use crate::utils::*;

use self::events::{EventKind, Events, ObjEvent};
use self::extractables::Extractables;
use self::new_obj::NewObj;
use self::objects::*;
use self::save::{CanLoad, CanSave, Load, Save};
use self::sectors::*;
use self::wares::*;
use crate::game::actions::Actions;
use crate::game::commands::{CommandMine, Commands};
use crate::game::navigations::Navigations;
use std::borrow::BorrowMut;
use crate::game::station::Station;

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

// TODO: add tick to game field
// TODO: remove most of fields?
pub struct Game<'a, 'b> {
    pub total_time: TotalTime,
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

impl<'a, 'b> Game<'a, 'b> {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut dispatcher_builder = DispatcherBuilder::new();

        Locations::init_world(&mut world, &mut dispatcher_builder);
        Actions::init_world(&mut world, &mut dispatcher_builder);
        Commands::init_world(&mut world, &mut dispatcher_builder);
        Navigations::init_world(&mut world, &mut dispatcher_builder);
        Extractables::init_world(&mut world, &mut dispatcher_builder);
        Objects::init_world(&mut world, &mut dispatcher_builder);
        Cargos::init_world(&mut world, &mut dispatcher_builder);

        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(&mut world);

        Game {
            total_time: TotalTime(0.0),
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


        //        self.events.add_obj_event(ObjEvent::new(id, EventKind::Add));

        let entity = builder.build();

        info!("add_object {:?} from {:?}", entity, new_obj);

        entity
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        // update time
        self.total_time = self.total_time.add(delta_time);
        self.world.insert(delta_time);
        self.world.insert(self.total_time);
        info!("tick delta {} total {}", delta_time.0, self.total_time.0);

        // update systems
        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
    }

    pub fn save(&self, save: &mut impl Save) {}

    pub fn load(&mut self, load: &mut impl Load) {}

    // TODO: make sense?
    pub fn set_sectors(&mut self, sectors: Sectors) {
        info!("set_sectors");
        self.world.insert(sectors);
    }
}
