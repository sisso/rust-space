use space_flap::{Id, SpaceGame, WareData};

#[derive(Copy, Clone, Debug)]
pub enum StateScreen {
    Sector(Id),
    Galaxy,
    Fleet(Id),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeSpeed {
    Pause,
    Normal,
}

pub struct State {
    pub game: SpaceGame,
    pub screen: StateScreen,
    pub wares: Vec<WareData>,
    pub selected_object: Option<Id>,
    pub time_speed: TimeSpeed,
}

impl State {
    pub fn new() -> Self {
        _ = env_logger::builder()
            .filter(None, log::LevelFilter::Info)
            // .filter(Some("world_view"), log::LevelFilter::Warn)
            // .filter(Some("space_flap"), log::LevelFilter::Warn)
            // .filter(Some("space_domain"), log::LevelFilter::Warn)
            // .filter(Some("space_domain::conf"), log::LevelFilter::Debug)
            // .filter(Some("space_domain::game::loader"), log::LevelFilter::Warn)
            .try_init()
            .or_else(|err| {
                log::warn!("fail to initialize log {err:?}");
                Err(err)
            });

        let mut game = SpaceGame::new(vec![
            "--size".to_string(),
            "2".to_string(),
            "--fleets".to_string(),
            "0".to_string(),
        ]);

        let sector_id = game
            .get_sectors()
            .get(0)
            .expect("game has no sector")
            .get_id();

        let wares = game.list_wares();

        let state = State {
            game,
            screen: StateScreen::Sector(sector_id),
            time_speed: TimeSpeed::Normal,
            selected_object: None,
            wares,
        };

        state
    }
}

#[cfg(test)]
mod test {
    use crate::state::State;
    use log::LevelFilter::Debug;

    #[test]
    fn test1() {
        State::new();
    }
}
