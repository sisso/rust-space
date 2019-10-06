use specs::{Builder, Component as SpecComponent, DenseVecStorage, Entities, Entity, HashMapStorage, LazyUpdate, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage, EntityBuilder, SystemData};

use crate::utils::*;

//use self::actions::*;
//use self::commands::*;
use self::extractables::Extractables;
use self::locations::{Locations, Location};
use self::new_obj::NewObj;
use self::objects::*;
use self::save::{CanLoad, CanSave, Load, Save};
use self::sectors::*;
use self::wares::*;
use self::events::{Events, ObjEvent, EventKind};
use crate::game::extractables::Extractable;
use std::collections::HashMap;
use crate::game::locations::{Moveable, LocationSpace, LocationDock};
use specs::world::Generation;

pub mod sectors;
pub mod objects;
pub mod wares;
//pub mod actions;
//pub mod commands;
pub mod locations;
pub mod extractables;
pub mod save;
pub mod new_obj;
pub mod jsons;
pub mod ship;
//pub mod factory;
pub mod events;
//pub mod ai_high;


pub struct Tick {
    total_time: TotalTime,
    delta_time: DeltaTime
}

pub struct Game {
    world: World,
//    pub commands: Commands,
//    pub actions: Actions,
    pub sectors: Sectors,
    pub objects: Objects,
    pub locations: Locations,
    pub extractables: Extractables,
    pub cargos: Cargos,
    pub events: Events,
}

trait BuilderExtra {
    fn set<C: SpecComponent + Send + Sync>(&mut self, c: C) ;
}

impl<'a> BuilderExtra for EntityBuilder<'a> {
    /// Inserts a component for this entity.
    ///
    /// If a component was already associated with the entity, it will
    /// overwrite the previous component.
    #[inline]
    fn set<T: SpecComponent>(&mut self, c: T)  {
        {
            let mut storage: WriteStorage<T> = SystemData::fetch(&self.world);
            // This can't fail.  This is guaranteed by the lifetime 'a
            // in the EntityBuilder.
            storage.insert(self.entity, c).unwrap();
        }
    }
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();

        locations::init_world(&mut world);
        extractables::init_world(&mut world);
        objects::init_world(&mut world);
        wares::init_world(&mut world);

        Game {
            world,
//            commands: Commands::new(),
//            actions: Actions::new(),
            sectors: Sectors::new(),
            objects: Objects::new(),
            locations: Locations::new(),
            extractables: Extractables::new(),
            cargos: Cargos::new(),
            events: Events::new(),
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

        for location in new_obj.location {
            match location {
                Location::Space { sector_id, pos} => {
                    builder.set(LocationSpace { sector_id: sector_id, pos: pos });
                },
                Location::Docked { docked_id } => {
                    builder.set(LocationDock { docked_id: docked_id });
                },
            }
        };

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
//
//        self.events.add_obj_event(ObjEvent::new(id, EventKind::Add));
//
//        id

        builder.build()
    }

    pub fn tick(&mut self, total_time: TotalTime, delta_time: DeltaTime) {
        info!("game", &format!("tick delta {} total {}", delta_time.0, total_time.0));
        let tick = Tick { total_time, delta_time };
//        self.commands.execute(&tick, &self.objects, &self.extractables, &mut self.actions, &self.locations, &self.sectors, &mut self.cargos);
//        self.actions.execute(&tick, &self.sectors, &mut self.locations, &self.extractables, &mut self.cargos, &mut self.events);
    }

    pub fn save(&self, save: &mut impl Save) {
        self.sectors.save(save);
//        self.objects.save(save);
        self.locations.save(save);
        self.extractables.save(save);
        self.cargos.save(save);
//        self.actions.save(save);
//        self.commands.save(save);
    }

    pub fn load(&mut self, load: &mut impl Load) {
        self.sectors.load(load);
//        self.objects.load(load);
        self.locations.load(load);
        self.extractables.load(load);
        self.cargos.load(load);
//        self.commands.load(load);
//        self.actions.load(load);
    }
}
