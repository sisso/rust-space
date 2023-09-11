extern crate space_domain;

use space_domain::game::commands::Command;

use commons::math::P2;
use log::LevelFilter;
use space_domain::game;
use space_domain::game::building_site::BuildingSite;
use space_domain::game::label::Label;
use space_domain::game::loader::Loader;
use space_domain::game::sceneries;
use space_domain::game::scenery_random::{InitialCondition, RandomMapCfg};
use space_domain::game::station::Station;
use space_domain::game::wares::WareAmount;
use space_domain::game::Game;
use space_domain::utils::DeltaTime;
use specs::prelude::*;
use specs::WorldExt;
use std::borrow::Borrow;

#[test]
fn test_game_should_mine_and_deliver_cargo_to_shipyard_until_produce_a_new_ship() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    let mut game = Game::new();
    let _ = sceneries::load_basic_scenery(&mut game);

    tick_eventually(&mut game, |game| {
        let total_commands = game.world.read_storage::<Command>().borrow().count();
        total_commands > 2
    });
}

#[test]
fn test_construction_yard_should_be_build_by_miners_delivering_components() {
    let mut game = Game::new();
    let bs = sceneries::load_minimum_scenery(&mut game);

    // add stations prefab
    let station_code = "dummy_station";
    let new_obj = Loader::new_station().with_label(station_code.to_string());
    let station_prefab_id = Loader::add_prefab(&mut game.world, station_code, new_obj);

    // add building site
    _ = Loader::add_object(
        &mut game.world,
        &Loader::new_station_building_site(
            station_prefab_id,
            vec![WareAmount {
                ware_id: bs.ware_ore_id,
                amount: 10,
            }],
        )
        .at_position(bs.sector_0, P2::ZERO),
    );

    // add miner to extract ore into the building site
    _ = Loader::add_object(
        &mut game.world,
        &Loader::new_ship(1.0, "miner".to_string())
            .with_command(Command::mine())
            .at_position(bs.sector_0, P2::ZERO),
    );

    // wait until building site is complete
    tick_eventually(&mut game, |game| {
        game.world.read_storage::<BuildingSite>().borrow().count() == 0
    });

    // check the new station was created
    let labels = game.world.read_storage::<Label>();
    let stations = game.world.read_storage::<Station>();
    let new_station_created = (&labels, &stations)
        .join()
        .find(|(l, _)| l.label == station_code)
        .is_some();
    assert!(new_station_created);
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

fn tick_eventually(game: &mut Game, expected_check: fn(game: &mut Game) -> bool) {
    let delta = DeltaTime(0.5);
    for _tick in 0..300 {
        game.tick(delta);
        if expected_check(game) {
            return;
        }
    }

    panic!("max tickets completed without desired result");
}
