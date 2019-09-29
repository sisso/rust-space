use crate::game::Game;
use std::time::Duration;
use crate::utils::Seconds;

pub struct GameApi {
    game: Game,
    total_time: f32,
}

/// Represent same interface we intend to use through FFI
/// TODO: ^
impl GameApi {
    pub fn new() -> Self {
        GameApi {
            game: Game::new(),
            total_time: 0.0
        }
    }

    pub fn new_game(&mut self) {
        crate::local_game::init_new_game(&mut self.game);
    }

    pub fn update(&mut self, elapsed: Duration) {
        let delta = (elapsed.as_millis() as f32) / 1000.0;
        self.total_time += delta;
        self.game.tick(Seconds(self.total_time), Seconds(delta))
    }

    /// TODO: remove this method, should not be used directly
    pub fn get_game(&self) -> &Game {
        &self.game
    }
}

