extern crate space_domain;

use space_domain::game::objects::ObjId;
use space_domain::game::wares::{Cargo};
use space_domain::utils::{DeltaTime};
use specs::WorldExt;
use space_domain::game::loader::Loader;
use space_domain::game::Game;
use space_domain::game::commands::Command;
use std::borrow::Borrow;

fn load_objects(game: &mut Game) -> (ObjId, ObjId) {
    let scenery = Loader::load_basic_scenery(game);
    (scenery.shipyard_id, scenery.miner_id)
}

#[test]
fn test_game_should_mine_and_deliver_cargo_to_station() {
    let mut game = Game::new();
    let _ = load_objects(&mut game);

    let delta = DeltaTime(0.5);

    for tick in 0..300 {
        game.tick(delta);

        let total_commands =
            game.world.read_storage::<Command>()
                .borrow()
                .count();

        if total_commands > 2 {
            // we have new ship

            // assert that we don't start with this ship
            assert!(tick >= 1);

            return;
        }
    }

    panic!("looks like station never have cargo");
}
