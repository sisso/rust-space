extern crate domain_b;

use domain_b::game::commands::Command;


use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use commons::math::P2;
use domain_b::game;
use domain_b::game::bevy_utils::WorldExt;
use domain_b::game::building_site::BuildingSite;
use domain_b::game::label::Label;
use domain_b::game::loader::Loader;
use domain_b::game::sceneries;
use domain_b::game::scenery_random::{InitialCondition, RandomMapCfg};
use domain_b::game::station::Station;
use domain_b::game::utils::{DeltaTime, Speed};
use domain_b::game::wares::WareAmount;
use domain_b::game::Game;

#[test]
fn test_game_should_mine_and_deliver_cargo_to_mothership_until_produce_a_new_ship() {
    let mut game = Game::new();
    _ = sceneries::load_basic_mothership_scenery(&mut game);
    game.debug_dump();

    tick_eventually(&mut game, |game| count_commands(game) > 1);
}

#[test]
fn test_game_should_mine_and_deliver_cargo_to_shipyard_until_produce_a_new_ship() {
    let mut game = Game::new();
    sceneries::load_basic_scenery(&mut game);
    game.debug_dump();

    tick_eventually(&mut game, |game| count_commands(game) > 2);
}

fn count_commands(game: &mut Game) -> usize {
    Loader::count_by_component::<Command>(&mut game.world)
}

#[test]
fn test_construction_yard_should_be_build_by_miners_delivering_components() {
    let mut game = Game::new();
    let bs = sceneries::load_minimum_scenery(&mut game);

    // add stations prefab
    let station_code = "dummy_station";
    let new_obj = Loader::new_station().with_label(station_code.to_string());

    game.world.run_commands(|mut commands| {
        let station_prefab_id = Loader::add_prefab(
            &mut commands,
            station_code,
            "Dummy Station",
            new_obj,
            false,
            true,
        );

        // add building site
        _ = Loader::add_object(
            &mut commands,
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
            &mut commands,
            &Loader::new_ship(1.0, "miner".to_string())
                .with_command(Command::mine())
                .at_position(bs.sector_0, P2::ZERO),
        );
    });

    // wait until building site is complete
    tick_eventually(&mut game, |game| {
        Loader::count_by_component::<BuildingSite>(&mut game.world) == 0
    });

    // check the new station was created
    let mut ss: SystemState<Query<(&Label, &Station)>> = SystemState::new(&mut game.world);
    let query = ss.get(&mut game.world);

    assert!(query
        .iter()
        .find(|(label, _)| label.label == station_code)
        .is_some());
}

#[test]
fn test_load_random_scenery() {
    let mut game = Game::new();

    let path = "../data/game.conf";
    let content = std::fs::read_to_string(path).expect("fail to read config file");
    let cfg = game::conf::load_str(&content).unwrap();

    game.world
        .run_commands(|mut commands| game::loader::load_prefabs(&mut commands, &cfg.prefabs));

    game::scenery_random::load_random(
        &mut game,
        &RandomMapCfg {
            size: (2, 2),
            seed: 0,
            fleets: 2,
            universe_cfg: cfg.system_generator.unwrap(),
            initial_condition: InitialCondition::Minimal,
            params: cfg.params,
        },
    );

    // let start = time::Instant::now();
    //
    // // simulate tickets
    // for _ in 0..5_000 {
    //     game.tick(DeltaTime(0.1));
    // }
    //
    // let end = time::Instant::now();
    // println!("{:?}", end - start);
}

#[test]
fn test_mining_on_high_speed_with_orbiting_objects() {
    let mut game = Game::new();

    let rs = sceneries::load_basic_mothership_scenery(&mut game);

    let sun_id = game.world.run_commands(|mut commands| {
        Loader::add_object(&mut commands, &Loader::new_star(rs.sector_id))
    });

    Loader::set_obj_at_orbit(
        &mut game.world,
        rs.asteroid_id,
        sun_id,
        2.0,
        0.0,
        Speed(0.1),
    );

    let delta = DeltaTime(30.0);
    for _tick in 0..300 {
        game.tick(delta);
        if Loader::count_by_component::<Command>(&mut game.world) > 1 {
            return;
        }
    }
    panic!("fail to create a fleet on timer end")
}

fn tick_eventually(game: &mut Game, expected_check: fn(game: &mut Game) -> bool) {
    let delta = DeltaTime(0.5);
    for _tick in 0..500 {
        game.tick(delta);
        if expected_check(game) {
            return;
        }
    }

    panic!("max tickets completed without desired result");
}
