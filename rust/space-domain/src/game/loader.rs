use crate::game::wares::WareId;
use crate::game::sectors::{SectorId, SectorsIndex, Sector, Jump, JumpId};
use crate::game::objects::ObjId;
use crate::game::Game;
use crate::game::new_obj::NewObj;
use crate::game::extractables::Extractable;
use crate::utils::{V2, Speed, Position};
use crate::game::commands::Commands;
use specs::{WorldExt, Builder};
use crate::game::locations::Location;
use crate::game::events::{Event, EventKind};
use std::borrow::BorrowMut;

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
        // init wares
        let ware_ore_id = WareId(0);

        // init sectors
        let sector_0 = Loader::new_sector(game);
        let sector_1 = Loader::new_sector(game);

        Loader::new_jump(game, sector_0, V2::new(0.5, 0.3), sector_1, V2::new(0.0, 0.0));
        SectorsIndex::update_index_from_world(&mut game.world);

        // init objects
        let asteroid_id = Loader::new_asteroid(game, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let station_id = Loader::new_station(game, sector_0, V2::new(1.0, -3.0));
        let miner_id = Loader::new_ship_miner(game, station_id, 2.0);

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

    pub fn new_asteroid(game: &mut Game, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        game.add_object(
            NewObj::new()
                .extractable(Extractable { ware_id })
                .at_position(sector_id, pos),
        )
    }

    pub fn new_station(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
        game.add_object(
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_factory()
                .has_dock(),
        )
    }

    pub fn new_ship_miner(game: &mut Game, docked_at: ObjId, speed: f32) -> ObjId {
        game.add_object(
            NewObj::new()
                .with_cargo(2.0)
                .with_speed(Speed(speed))
                .at_dock(docked_at)
                .can_dock()
                .with_ai()
                .with_command_mine(),
        )
    }

    pub fn new_sector(game: &mut Game) -> ObjId {
        game.add_object(NewObj::new().with_sector())
    }

    pub fn new_jump(game: &mut Game, from_sector_id: SectorId, from_pos: Position, to_sector_id: JumpId, to_pos: Position) -> (ObjId, ObjId) {
        let jump_from_id = game.world.create_entity()
            .with(Location::Space { pos: from_pos, sector_id: from_sector_id })
            .build();

        let jump_to_id = game.world.create_entity()
            .with(Location::Space { pos: to_pos, sector_id: to_sector_id })
            .with(Jump { target_id: jump_from_id })
            .build();

        game.world.write_storage::<Jump>()
            .borrow_mut()
            .insert(jump_from_id, Jump { target_id: jump_to_id })
            .unwrap();

        game.world.create_entity().with(Event::new(jump_from_id, EventKind::Add)).build();
        game.world.create_entity().with(Event::new(jump_to_id, EventKind::Add)).build();

        info!("{:?} creating jump from {:?} to {:?}", jump_from_id, from_sector_id, to_sector_id);
        info!("{:?} creating jump from {:?} to {:?}", jump_to_id, to_sector_id, from_sector_id);

        (jump_from_id, jump_to_id)
    }
}