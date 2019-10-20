extern crate space_domain;
extern crate space_macros;

use space_domain::game_api::GameApi;
use std::time::Duration;
use space_macros::{log, info};

fn main() -> Result<(), std::io::Error> {
//    space_domain::local_game::run();
    info!(target: "main", "--------------------------------------------------");
    info!(target: "main", "start");
    info!(target: "main", "--------------------------------------------------");

    let mut game_api = GameApi::new();
    game_api.new_game();

    let time_rate = Duration::from_millis(1000);

    for _ in 0..50 {
        info!(target: "main", "--------------------------------------------------");

        game_api.set_inputs(&vec![]);
        game_api.update(time_rate);
        game_api.get_inputs(move |bytes| {
            info!("receive {:?}", bytes)
        });
    }

    Ok(())
}
