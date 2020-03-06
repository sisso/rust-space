use crate::specs_extras::*;
use crate::game::wares::{WareId, Cargo, WareAmount};
use crate::game::sectors::{SectorId, SectorsIndex, Sector, Jump, JumpId};
use crate::game::objects::ObjId;
use crate::game::Game;
use crate::game::new_obj::NewObj;
use crate::game::extractables::Extractable;
use crate::utils::{V2, Speed, Position, DeltaTime};
use crate::game::commands::{Commands, MineState, Command};
use specs::{WorldExt, Builder, World};
use crate::game::locations::{Location, Moveable};
use crate::game::events::{Event, EventKind, Events};
use std::borrow::BorrowMut;
use crate::game::dock::HasDock;
use crate::game::shipyard::Shipyard;
use crate::game::station::Station;
use crate::game::factory::{Factory, Production};
use crate::game::order::Order;

pub struct Loader {

}

pub struct BasicScenery {
    pub asteroid_id: ObjId,
    pub shipyard_id: ObjId,
    pub miner_id: ObjId,
    pub ware_ore_id: WareId,
    pub ware_components_id: WareId,
    pub sector_0: SectorId,
    pub sector_1: SectorId,
    pub component_factory_id: ObjId,
}

impl Loader {
    /// Basic scenery used for testing and samples
    ///
    /// Is defined as a simple 2 sector, one miner ship, a station and asteroid
    pub fn load_basic_scenery(game: &mut Game) -> BasicScenery {
        let world = &mut game.world;

        // init wares
        let ware_ore_id = Loader::new_ware(world);
        let ware_components_id = Loader::new_ware(world);

        // init sectors
        let sector_0 = Loader::new_sector(world);
        let sector_1 = Loader::new_sector(world);

        Loader::new_jump(world, sector_0, V2::new(0.5, 0.3), sector_1, V2::new(0.0, 0.0));
        SectorsIndex::update_index_from_world(world);

        // init objects
        let asteroid_id = Loader::new_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let component_factory_id = Loader::new_factory(world,
           sector_0,
           V2::new(3.0, -1.0),
           vec![WareAmount(ware_ore_id, 2.0)],
           vec![WareAmount(ware_components_id, 1.0)],
            DeltaTime(1.0),
        );
        let shipyard_id = Loader::new_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
        let miner_id = Loader::new_ship_miner(world, shipyard_id, 2.0);

        // return scenery
        BasicScenery {
            asteroid_id,
            shipyard_id,
            miner_id,
            ware_ore_id,
            ware_components_id,
            sector_0,
            sector_1,
            component_factory_id,
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

    pub fn new_shipyard(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_shipyard(Shipyard::new(WareAmount(ware_id, 5.0), DeltaTime(5.0)))
                .has_dock(),
        )
    }

    pub fn new_factory(world: &mut World, sector_id: SectorId, pos: V2, input: Vec<WareAmount>, output: Vec<WareAmount>, time: DeltaTime) -> ObjId {
        let production = Production {
            input,
            output,
            time
        };

        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_factory(Factory::new(production))
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
                .with_command(Command::mine()),
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

    // TODO: receive new obj or reference?
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

        for speed in &new_obj.speed {
            builder.set(Moveable { speed: speed.clone() });
        }

        for extractable in &new_obj.extractable {
            builder.set(extractable.clone());
        }

        if new_obj.station {
            builder.set(Station {});
        }

        if new_obj.sector {
            builder.set(Sector {});
        }

        for target_id in new_obj.jump_to {
            builder.set(Jump { target_id });
        }

        for command in &new_obj.command {
            builder.set(command.clone());
        }

        for shipyard in &new_obj.shipyard {
            builder.set(shipyard.clone());
            builder.set(Order::WareRequest {
                wares_id: vec![shipyard.input.get_ware_id()]
            })
        }

        if new_obj.cargo_size > 0.0 {
            let mut cargo = Cargo::new(new_obj.cargo_size);
            // TODO: shipyards?
            for factory in &new_obj.factory {
                factory.setup_cargo(&mut cargo);
            }
            builder.set(cargo);
        }

        for factory in &new_obj.factory {
            builder.set(factory.clone());
            builder.set(Order::WareRequest {
                wares_id: factory.production.request_wares_id()
            })
        }

        let entity = builder.build();

        info!("add_object {:?} from {:?}", entity, new_obj);

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(entity, EventKind::Add));

        entity
    }
}