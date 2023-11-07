use space_flap::{Id, SpaceGame, WareData};

#[derive(Copy, Clone, Debug)]
pub enum StateScreen {
    Sector(Id),
    SectorPlot { sector_id: Id, plot_id: Id },
    Obj(Id),
}

impl StateScreen {}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeSpeed {
    Pause,
    Normal,
}

pub struct State {
    pub game: SpaceGame,
    pub screen: StateScreen,
    pub wares: Vec<WareData>,
    pub time_speed: TimeSpeed,
}

impl State {
    pub fn new() -> Self {
        log::info!("initializing game");

        _ = env_logger::builder()
            .filter(None, log::LevelFilter::Info)
            // .filter(Some("world_view"), log::LevelFilter::Warn)
            // .filter(Some("space_flap"), log::LevelFilter::Warn)
            // .filter(Some("space_domain"), log::LevelFilter::Warn)
            // .filter(Some("space_domain::conf"), log::LevelFilter::Debug)
            .filter(Some("space_domain::game::loader"), log::LevelFilter::Trace)
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
            "--seed".to_string(),
            "1".to_string(),
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
            wares,
        };

        state
    }

    pub fn get_current_sector_id(&self) -> Id {
        match &self.screen {
            StateScreen::Sector(sector_id) => *sector_id,
            StateScreen::Obj(id) => {
                let sector_id = self
                    .game
                    .get_obj_coords(*id)
                    .map(|coords| coords.get_sector_id());

                return sector_id.expect("selected object has no sector_id");
            }
            StateScreen::SectorPlot { sector_id, .. } => *sector_id,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::state::State;

    #[test]
    fn test1() {
        State::new();
    }
}
