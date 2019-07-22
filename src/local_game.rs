use crate::game::*;
use crate::game::sectors::*;
use crate::game::objects::*;
use crate::game::wares::*;
use crate::game::commands::*;
use crate::utils::*;

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
                    time: Seconds(1.0),
                }
            )
            .at_position(sector_id, pos)
    )
}

fn new_station(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(100)
            .with_speed(Speed(1.0))
            .at_position(sector_id, pos)
            .has_dock()
    )
}

fn new_ship_miner(game: &mut Game, docked_at: ObjId) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(1)
            .with_speed(Speed(1.0))
            .at_dock(docked_at)
            .can_dock()
    )
}

pub fn run() {
    let mut game = Game::new();

    load_sectors(&mut game);
    load_objects(&mut game);

    for i in 0..10 {
        game.tick(Seconds(i as f32), Seconds(1.0));
    }
}
