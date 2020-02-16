extern crate space_domain;

use space_domain::game::extractables::Extractable;
use space_domain::game::locations::Location;

use space_domain::game::new_obj::NewObj;
use space_domain::game::objects::ObjId;
use space_domain::game::sectors::test_scenery;
use space_domain::game::sectors::SectorId;
use space_domain::game::wares::{WareId, Cargo};
use space_domain::game::Game;
use space_domain::test::assert_v2;
use space_domain::utils::{DeltaTime, Speed, V2};
use specs::WorldExt;
use std::borrow::Borrow;
use space_domain::game::loader::Loader;

fn load_objects(game: &mut Game) -> (ObjId, ObjId) {
    let scenery = Loader::load_basic_scenery(game);
    (scenery.station_id, scenery.miner_id)
}

#[test]
fn test_game_should_mine_and_deliver_cargo_to_station() {
    let mut game = Game::new();

    let sectors = test_scenery::new_test_sectors();
    game.set_sectors(sectors);

    let (station_id, ship_id) = load_objects(&mut game);

    let delta = DeltaTime(0.5);

    for _ in 0..100 {
        game.tick(delta);
    }

    let cargo_storage = &game.world.read_storage::<Cargo>();
    let station_cargo = cargo_storage.get(station_id).unwrap();
    assert!(station_cargo.get_total() > 0.0);
}
