extern crate space_domain;

use space_domain::game::commands::Command;

use space_domain::game;
use space_domain::game::sceneries;
use space_domain::game::scenery_random::{InitialCondition, RandomMapCfg};
use space_domain::game::Game;
use space_domain::utils::DeltaTime;
use specs::WorldExt;
use std::borrow::Borrow;

fn load_objects(game: &mut Game) {
    sceneries::load_advanced_scenery(&mut game.world);
}

#[test]
fn test_game_should_mine_and_deliver_cargo_to_station() {
    let mut game = Game::new();
    load_objects(&mut game);

    let delta = DeltaTime(0.5);

    for tick in 0..300 {
        game.tick(delta);

        let total_commands = game.world.read_storage::<Command>().borrow().count();

        if total_commands > 2 {
            // we have new ship

            // assert that we don't start with this ship
            assert!(tick >= 1);

            return;
        }
    }

    panic!("we never produce any ship");
}

#[test]
fn test_load_random_scenery() {
    let mut game = Game::new();

    let path = "../data/game.conf";
    let content = std::fs::read_to_string(path).expect("fail to read config file");
    let cfg = game::conf::load_str(&content).unwrap();

    game::loader::load_prefabs(&mut game.world, &cfg.prefabs);

    game::scenery_random::load_random(
        &mut game,
        &RandomMapCfg {
            size: 2,
            seed: 0,
            fleets: 2,
            universe_cfg: cfg.system_generator.unwrap(),
            initial_condition: InitialCondition::Minimal,
            params: cfg.params,
        },
    );
}
