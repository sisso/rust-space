extern crate space_domain;

use space_domain::game::extractables::Extractable;
use space_domain::game::locations::{LocationSector, LocationSpace};
use space_domain::game::navigations::Navigation;
use space_domain::game::new_obj::NewObj;
use space_domain::game::objects::ObjId;
use space_domain::game::sectors::test_scenery;
use space_domain::game::sectors::SectorId;
use space_domain::game::wares::WareId;
use space_domain::game::Game;
use space_domain::test::assert_v2;
use space_domain::utils::{DeltaTime, Speed, TotalTime, V2};
use specs::WorldExt;
use std::borrow::Borrow;

const WARE_ORE: WareId = WareId(0);

fn new_asteroid(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .extractable(Extractable {
                ware_id: WARE_ORE,
                time: DeltaTime(1.5),
            })
            .at_sector(sector_id)
            .at_position(pos),
    )
}

fn new_station(game: &mut Game, sector_id: SectorId, pos: V2) -> ObjId {
    game.add_object(
        NewObj::new()
            .with_cargo(100.0)
            .at_sector(sector_id)
            .at_position(pos)
            .has_dock(),
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
            .with_ai(),
    )
}

fn load_objects(game: &mut Game) -> (ObjId, ObjId) {
    // asteroid field
    let _ = new_asteroid(game, test_scenery::SECTOR_1, V2::new(-5.0, 5.0));

    // station
    let station_id = new_station(game, test_scenery::SECTOR_0, V2::new(5.0, -5.0));

    // miner
    let ship_id = new_ship_miner(game, test_scenery::SECTOR_0, station_id, 2.0);

    space_domain::game::commands::set_command_mine(&mut game.world, ship_id);

    (station_id, ship_id)
}

#[test]
fn test_game_should_run() {
    let mut game = Game::new();

    let sectors = test_scenery::new_test_sectors();
    game.set_sectors(sectors);

    let (_station_id, ship_id) = load_objects(&mut game);

    let delta = DeltaTime(0.5);

    for _ in 0..100 {
        game.tick(delta);
    }

    let sector_id = game
        .world
        .read_storage::<LocationSector>()
        .borrow()
        .get(ship_id)
        .unwrap()
        .sector_id;
    let pos = game
        .world
        .read_storage::<LocationSpace>()
        .borrow()
        .get(ship_id)
        .unwrap()
        .pos;
    assert_eq!(sector_id, test_scenery::SECTOR_1);
    assert_v2(pos, V2::new(-5.0, 5.0));
}
