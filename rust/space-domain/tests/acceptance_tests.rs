extern crate space_domain;

use space_domain::game::Game;
use space_domain::utils::{DeltaTime, TotalTime, V2, Seconds, Speed};
use space_domain::game::sectors::SectorId;
use space_domain::game::objects::ObjId;
use space_domain::game::new_obj::NewObj;
use space_domain::game::extractables::Extractable;
use space_domain::game::wares::WareId;
use space_domain::game::sectors::test_scenery;
use space_domain::game::commands::Command;

//use space_domain::game::objects::ObjId;
//use space_domain::utils::{Seconds, V2, DeltaTime, TotalTime};
//use space_domain::game::sectors::SectorId;
//
//struct BasicScenary {
//    game: Game
//}
//
//impl BasicScenary {
//    pub fn new() -> Self {
//        let mut game = Game::new();
////        space_domain::local_game::init_new_game(&mut game);
//        BasicScenary {
//            game
//        }
//    }
//
//    pub fn get_ship(&self) -> ObjId {
////        let mut ships: Vec<ObjId> = self.game.objects.list().filter_map(|obj| {
////            self.game.locations.get_speed(&obj.id).map(|_| obj.id.clone())
////        }).collect();
////
////        ships.remove(0)
//        unimplemented!();
//    }
//}
//
//#[test]
//fn test_jump_should_warp_into_correct_position() {
//    let mut scenary = BasicScenary::new();
//    let obj_id = scenary.get_ship();
//    for i in 0..100 {
//        let total_time = TotalTime(i as f64);
//        scenary.game.tick(total_time, DeltaTime(0.5));
//
//        let location = scenary.game.locations.get_location(&obj_id);
//
//        match location {
//            Some(Location::Space { sector_id, pos }) => {
//                if *sector_id == SectorId(1) {
//                    // when ship jump into sector 2, it should be into this position
//                    let distance = V2::new(5.0, -5.0).sub(pos).length();
//                    assert!(distance < 0.1, format!("unexpected distance {:?}, ship position {:?}", distance, pos));
//
//                    // we are good
//                    return;
//                }
//            },
//            _ => {}
//        }
//    }
//
//    assert!(false, "ship not jump into time");
//}

const WARE_ORE: WareId = WareId(0);

fn new_asteroid(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .extractable(
                Extractable {
                    ware_id: WARE_ORE,
                    time: DeltaTime(1.5),
                }
            )
            .at_sector(sector_id)
            .at_position(pos)
    )
}

fn new_station(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(100.0)
            .at_sector(sector_id)
            .at_position(pos)
            .has_dock()
    )
}

fn new_ship_miner(game: &mut Game, sector_id: SectorId, docked_at: ObjId, speed: f32) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(2.0)
            .with_speed(Speed(speed))
            .at_sector(sector_id)
            .at_dock(docked_at)
            .can_dock()
            .with_ai()
    )
}

fn load_objects(game: &mut Game) {
    // asteroid field
    let _ = new_asteroid(game, test_scenery::SECTOR_1, V2::new(-5.0, 5.0));

    // station
    let station_id = new_station(game, test_scenery::SECTOR_0, V2::new(5.0, -5.0));

    // miner
    let ship_id = new_ship_miner(game, test_scenery::SECTOR_0, station_id, 2.0);

    game.set_command(ship_id, Command::Mine);

//    for _ in 0..10 {
//        let speed: f32 = rand::thread_rng().gen_range(1.0, 3.0);
//        let ship_id = new_ship_miner(game, station_id, speed);
//        game.set_command(ship_id, Command::Mine);
//    }
}

#[test]
fn test_game_should_run() {
    let mut game = Game::new();

    let sectors = test_scenery::new_test_sectors();
    game.set_sectors(sectors);

    load_objects(&mut game);

    let mut total = TotalTime(0.0);
    let delta = DeltaTime(0.1);

    for i in 0..100 {
        game.tick(total, delta);
        total = total.add(delta);
    }

    unimplemented!();
}
