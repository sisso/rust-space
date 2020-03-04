use crate::specs_extras::*;
use crate::game::wares::{WareId, Cargo};
use crate::game::sectors::{SectorId, SectorsIndex, Sector, Jump, JumpId};
use crate::game::objects::ObjId;
use crate::game::Game;
use crate::game::new_obj::NewObj;
use crate::game::extractables::Extractable;
use crate::utils::{V2, Speed, Position};
use crate::game::commands::{Commands, CommandMine};
use specs::{WorldExt, Builder, World};
use crate::game::locations::{Location, Moveable};
use crate::game::events::{Event, EventKind, Events};
use std::borrow::BorrowMut;
use crate::game::dock::HasDock;
use crate::game::shipyard::Shipyard;

pub struct Loader {

}

pub struct BasicScenery {
    pub asteroid_id: ObjId,
    pub station_id: ObjId,
    pub miner_id: ObjId,
    pub ware_ore_id: WareId,
    pub sector_0: SectorId,
    pub sector_1: SectorId,
}

impl Loader {
    /// Basic scenery used for testing and samples
    ///
    /// Is defined as a simple 2 sector, one miner ship, a station and asteroid
    pub fn load_basic_scenery(game: &mut Game) -> BasicScenery {
        let world = &mut game.world;

        // init wares
        let ware_ore_id = Loader::new_ware(world);

        // init sectors
        let sector_0 = Loader::new_sector(world);
        let sector_1 = Loader::new_sector(world);

        Loader::new_jump(world, sector_0, V2::new(0.5, 0.3), sector_1, V2::new(0.0, 0.0));
        SectorsIndex::update_index_from_world(world);

        // init objects
        let asteroid_id = Loader::new_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let station_id = Loader::new_station(world, sector_0, V2::new(1.0, -3.0));
        let miner_id = Loader::new_ship_miner(world, station_id, 2.0);

        // return scenery
        BasicScenery {
            asteroid_id,
            station_id,
            miner_id,
            ware_ore_id,
            sector_0,
            sector_1,
        }

    }

    pub fn new_asteroid(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .extractable(Extractable { ware_id })
                .at_position(sector_id, pos),
        )
    }

    pub fn new_station(world: &mut World, sector_id: SectorId, pos: V2) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_shipyard()
                .has_dock(),
        )
    }

    pub fn new_ship_miner(world: &mut World, docked_at: ObjId, speed: f32) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(2.0)
                .with_speed(Speed(speed))
                .at_dock(docked_at)
                .can_dock()
                .with_ai()
                .with_command_mine(),
        )
    }

    pub fn new_sector(world: &mut World) -> ObjId {
        Loader::add_object(
            world,
        NewObj::new().with_sector()
        )
    }

    pub fn new_ware(world: &mut World) -> WareId {
        Loader::add_object(
            world,
            NewObj::new().with_ware()
        )
    }

    pub fn new_jump(world: &mut World, from_sector_id: SectorId, from_pos: Position, to_sector_id: JumpId, to_pos: Position) -> (ObjId, ObjId) {
        let jump_from_id = world.create_entity()
            .with(Location::Space { pos: from_pos, sector_id: from_sector_id })
            .build();

        let jump_to_id = world.create_entity()
            .with(Location::Space { pos: to_pos, sector_id: to_sector_id })
            .with(Jump { target_id: jump_from_id })
            .build();

        world.write_storage::<Jump>()
            .borrow_mut()
            .insert(jump_from_id, Jump { target_id: jump_to_id })
            .unwrap();

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(jump_from_id, EventKind::Add));
        events.push(Event::new(jump_to_id, EventKind::Add));

        info!("{:?} creating jump from {:?} to {:?}", jump_from_id, from_sector_id, to_sector_id);
        info!("{:?} creating jump from {:?} to {:?}", jump_to_id, to_sector_id, from_sector_id);

        (jump_from_id, jump_to_id)
    }

    pub fn add_object(world: &mut World, new_obj: NewObj) -> ObjId {
        let mut builder = world.create_entity();

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

        if new_obj.station {
            // builder.set(Station {});
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

        if new_obj.shipyard {
            builder.set(Shipyard::new());
        }

        if new_obj.cargo_size > 0.0 {
            let mut cargo = Cargo::new(new_obj.cargo_size);
            if let Some(factory) = &new_obj.factory {
                factory.setup_cargo(&mut cargo);
            }
            builder.set(cargo);
        }

        let entity = builder.build();

        info!("add_object {:?} from {:?}", entity, new_obj);

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(entity, EventKind::Add));

        entity
    }
}