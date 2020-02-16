use crate::game::wares::WareId;
use crate::game::sectors::{SectorId, Sectors, Sector, Jump, JumpId};
use crate::game::objects::ObjId;
use crate::game::Game;
use crate::game::new_obj::NewObj;
use crate::game::extractables::Extractable;
use crate::utils::{V2, Speed, Position};
use crate::game::commands::Commands;

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
        let ware_ore_id = WareId(0);

        let sector_0: SectorId = SectorId(0);
        let sector_1: SectorId = SectorId(1);

        let jump_0_to_1: Jump = Jump {
            id: JumpId(0),
            sector_id: sector_0,
            pos: Position { x: 4.0, y: 0.0 },
            to_sector_id: sector_1,
            to_pos: Position { x: 0.0, y: 3.0 },
        };

        let jump_1_to_0: Jump = Jump {
            id: JumpId(1),
            sector_id: sector_1,
            pos: Position { x: 0.0, y: 3.0 },
            to_sector_id: sector_0,
            to_pos: Position { x: 4.0, y: 0.0 },
        };

        let mut sectors = Sectors::new();
        sectors.add_sector(Sector { id: sector_0 });
        sectors.add_sector(Sector { id: sector_1 });
        sectors.add_jump(jump_0_to_1.clone());
        sectors.add_jump(jump_1_to_0.clone());
        game.set_sectors(sectors);

        let asteroid_id = Loader::new_asteroid(game, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let station_id = Loader::new_station(game, sector_0, V2::new(1.0, -3.0));
        let miner_id = Loader::new_ship_miner(game, station_id, 2.0);

        Commands::set_command_mine(&mut game.world, miner_id);

        BasicScenery {
            asteroid_id,
            station_id,
            miner_id,
            ware_ore_id,
            sector_0,
            sector_1
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
                .with_ai(),
        )
    }
}