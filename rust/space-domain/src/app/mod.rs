use crate::game::game::{Game, NewGameParams};
use crate::game::save_manager::SaveManager;

pub struct App;

impl App {
    pub fn continue_last(save_manager: &mut SaveManager) -> Option<Game> {
        let last_save_game = save_manager
            .get_last()
            .expect("fail to read latest save game")?;
        let save_data = save_manager
            .read(&last_save_game.filename)
            .expect("fail to read save");
        let game = Game::load_from_string(save_data).expect("fail parse save data");
        Some(game)
    }

    pub fn start_new_game(params: NewGameParams) -> Game {
        Game::new(params)
    }
}
