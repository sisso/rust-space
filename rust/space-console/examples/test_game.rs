extern crate space_domain;

use log::info;
use space_domain::ffi::FFIApi;
use std::time::{Duration, Instant};

fn main() -> Result<(), std::io::Error> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    //    space_domain::local_game::run();
    log::info!(target: "main", "--------------------------------------------------");
    log::info!(target: "main", "start");
    log::info!(target: "main", "--------------------------------------------------");

    let mut game_api = FFIApi::new();
    game_api.new_game();

    let time_rate = Duration::from_millis(1000 / 60);

    loop {
        let start = Instant::now();
        game_api.set_inputs(&vec![]);
        game_api.update(time_rate);
        let game_update_time = Instant::now();
        let mut total_bytes = 0;
        game_api.get_inputs(|bytes| total_bytes += bytes.len());
        let input_time = Instant::now();

        let now = input_time;
        let delta = now - start;
        let wait_time = if delta > time_rate {
            Duration::new(0, 0)
        } else {
            time_rate - delta
        };

        log::info!(
            "gui - delta {:?}, wait_time: {:?}, ration: {:?}% usage",
            delta,
            wait_time,
            (100.0 / (time_rate.as_millis() as f64 / delta.as_millis() as f64)) as i32
        );
        log::debug!(
            "    - update {:?}, collect_inputs: {:?}",
            game_update_time - start,
            input_time - game_update_time
        );

        if delta < time_rate {
            std::thread::sleep(wait_time);
        } else {
            log::warn!(
                "gui - delta {:?}, wait_time: 0.0: missing time frame ofr {:?}",
                delta,
                time_rate
            );
        }
    }

    Ok(())
}
