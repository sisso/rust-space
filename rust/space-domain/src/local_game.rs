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
use rand::Rng;

const WARE_ORE: WareId = WareId(0);

const SECTOR_0: SectorId = SectorId(0);
const SECTOR_1: SectorId = SectorId(1);

fn load_sectors(game: &mut Game) {
    game.sectors.add_sector(Sector { id: SECTOR_0, });
    game.sectors.add_sector(Sector { id: SECTOR_1, });

    game.sectors.add_jump(Jump {
        id: JumpId(0),
        sector_id: SECTOR_0,
        pos: Position { x: -5.0, y: 5.0 },
        to_sector_id: SECTOR_1,
        to_pos: Position { x:  5.0, y: -5.0 }
    });

    game.sectors.add_jump(Jump {
        id: JumpId(1),
        sector_id: SECTOR_1,
        pos: Position { x: 5.0, y: -5.0 },
        to_sector_id: SECTOR_0,
        to_pos: Position { x:  -5.0, y: 5.0 }
    });
}

fn load_objects(game: &mut Game) {
    // asteroid field
    let _ = new_asteroid(game, SECTOR_1, V2::new(-5.0, 5.0));

    // station
    let station_id = new_station(game, SECTOR_0, V2::new(5.0, -5.0));

    // miner
    let ship_id = new_ship_miner(game, station_id, 2.0);
    game.commands.set_command(ship_id, Command::Mine);

//    for _ in 0..10 {
//        let speed: f32 = rand::thread_rng().gen_range(1.0, 3.0);
//        let ship_id = new_ship_miner(game, station_id, speed);
//        game.set_command(ship_id, Command::Mine);
//    }
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

fn new_ship_miner(game: &mut Game, docked_at: ObjId, speed: f32) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(2.0)
            .with_speed(Speed(speed))
            .at_dock(docked_at)
            .can_dock()
            .with_ai()
    )
}

pub fn init_new_game(game: &mut Game) {
    load_sectors(game);
    load_objects(game);
}

pub fn run() {
    let mut game = Game::new();

    let load_file =
//        Some("/tmp/01_24.json");
        None;

    match load_file {
        Some(file) => {
            load_from_save(&mut game, file);
        },
        None => {
            init_new_game(&mut game);
        }
    }

    for i in 0..50 {
        let total_time = Seconds(i as f32);
        game.tick(total_time, Seconds(1.0));
        assert_saves(&game, i);
    }
}

fn assert_saves(game: &Game, tick: u32) {
    // save
    let file_1 = format!("/tmp/01_{}.json", tick);
    let file_2 = format!("/tmp/02_{}.json", tick);

    {
        save_to_file(game,  tick, file_1.as_ref());
    }

    // load
    {
        let mut game = Game::new();
        load_from_save(&mut game, file_1.as_ref());
        save_to_file(&game,  tick, file_2.as_ref());
    }
}

fn save_to_file(game: &Game, tick: u32, file: &str) {
    let mut save = SaveToFile::new(file);
    save.add_header("game", json!({
            "total_time": tick
        }));
    game.save(&mut save);
    save.close();
}

fn load_from_save(game: &mut Game, load_file: &str) {
    let mut load = LoadFromFile::new(load_file);
    game.load(&mut load);
}
