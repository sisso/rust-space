extern crate space_domain;

use space_domain::game::objects::ObjId;
use space_domain::game::wares::{Cargo};
use space_domain::utils::{DeltaTime};
use specs::WorldExt;
use space_domain::game::loader::Loader;
use space_domain::game::Game;

fn load_objects(game: &mut Game) -> (ObjId, ObjId) {
    let scenery = Loader::load_basic_scenery(game);
    (scenery.shipyard_id, scenery.miner_id)
}

#[test]
fn test_game_should_mine_and_deliver_cargo_to_station() {
    let mut game = Game::new();
    let (station_id, _ship_id) = load_objects(&mut game);

    let delta = DeltaTime(0.5);

    for _ in 0..10 {
        game.tick(delta);

        let cargo_storage = &game.world.read_storage::<Cargo>();
        let station_cargo = cargo_storage.get(station_id).unwrap();
        if station_cargo.get_total() > 0.0 {
            // good
            return;
        }
    }

    panic!("looks like station never have cargo");
}
