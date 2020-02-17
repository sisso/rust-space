extern crate space_domain;
extern crate space_macros;

use space_domain::game_ffi::GameFFI;
use space_macros::{info, log};
use std::time::{Instant, Duration};
use std::thread::sleep;

fn main() -> Result<(), std::io::Error> {
    //    space_domain::local_game::run();
    info!(target: "main", "--------------------------------------------------");
    info!(target: "main", "start");
    info!(target: "main", "--------------------------------------------------");

    let mut game_api = GameFFI::new();
    game_api.new_game();

    let time_rate = Duration::from_millis(1000);
    let start = std::time::Instant::now();
    let wait_time = Duration::from_secs(1);

    loop {
        info!(target: "main", "--------------------------------------------------");

        game_api.set_inputs(&vec![]);
        game_api.update(time_rate);
        game_api.get_inputs(move |bytes| info!("receive {:?}", bytes));

        if start.elapsed() >= wait_time {
            break;
        }
    }

    Ok(())
}

