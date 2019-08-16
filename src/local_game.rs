use serde_json::json;

use crate::game::*;
use crate::game::sectors::*;
use crate::game::objects::*;
use crate::game::wares::*;
use crate::game::commands::*;
use crate::game::save::*;
use crate::utils::*;
use crate::game::extractables::Extractable;
use crate::game::new_obj::NewObj;

const WARE_ORE: WareId = WareId(0);

const SECTOR_0: SectorId = SectorId(0);
const SECTOR_1: SectorId = SectorId(1);

fn load_sectors(game: &mut Game) {
    game.add_sector(NewSector {
        id: SECTOR_0,
        jumps: vec![
            NewJump {
                to_sector_id: SECTOR_1,
                pos: Position { x: -5.0, y: 5.0 }
            }
        ]
    });

    game.add_sector(NewSector {
        id: SECTOR_1,
        jumps: vec![
            NewJump {
                to_sector_id: SECTOR_0,
                pos: Position { x: 5.0, y: -5.0 }
            }
        ]
    });
}

fn load_objects(game: &mut Game) {
    // asteroid field
    let _ = new_asteroid(game, SECTOR_1, V2::new(-5.0, 5.0));

    // station
    let station_id = new_station(game, SECTOR_0, V2::new(5.0, -5.0));

    // miner
    let ship_id = new_ship_miner(game, station_id);

    game.set_command(ship_id, Command::Mine);
}

fn new_asteroid(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .extractable(
                Extractable {
                    ware_id: WARE_ORE,
                    time: Seconds(1.5),
                }
            )
            .at_position(sector_id, pos)
    )
}

fn new_station(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(100.0)
            .at_position(sector_id, pos)
            .has_dock()
    )
}

fn new_ship_miner(game: &mut Game, docked_at: ObjId) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(2.0)
            .with_speed(Speed(1.0))
            .at_dock(docked_at)
            .can_dock()
            .with_ai()
    )
}

fn load_from_save(game: &mut Game, load_file: &str) {
    let mut load = LoadFromFile::new(load_file);
    game.load(&mut load);
}

pub fn run() {
    let mut game = Game::new();

    load_sectors(&mut game);
    load_objects(&mut game);

    for i in 0..50 {
        let total_time = Seconds(i as f32);
        game.tick(total_time, Seconds(1.0));
        let mut save = SaveToFile::new(&format!("/tmp/01_{}.json", i));
        save.init();
        save.add_header("game", json!({
            "total_time": i
        }));
        game.save(&mut save);
        save.close();
    }
}
