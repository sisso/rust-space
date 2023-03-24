use commons::math::Transform2;
use space_domain::game::{scenery_random, Game};
use specs::Entity;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub enum StateScreen {
    Sector(Entity),
    Galaxy,
    Fleet(Entity),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeSpeed {
    Pause,
    Normal,
}

pub struct State {
    pub game: Rc<RefCell<Game>>,
    pub screen: StateScreen,
    pub selected_sector: usize,
    pub selected_fleet: usize,
    pub selected_object: Option<Entity>,
    pub sector_view_transform: Transform2,
    pub time_speed: TimeSpeed,
}

impl State {
    pub fn new() -> Self {
        _ = env_logger::builder()
            .filter(None, log::LevelFilter::Warn)
            .filter(Some("world_view"), log::LevelFilter::Warn)
            .filter(Some("space_flap"), log::LevelFilter::Warn)
            .filter(Some("space_domain"), log::LevelFilter::Warn)
            .filter(Some("space_domain::game::loader"), log::LevelFilter::Warn)
            .try_init()
            .or_else(|err| {
                log::warn!("fail to initialize log {err:?}");
                Err(err)
            });

        let universe_cfg = space_domain::space_galaxy::system_generator::new_config_from_file(
            // TODO: remove abs path
            &PathBuf::from("/home/sisso/work/rust-space/rust/data/system_generator.conf"),
        );

        let mut game = Game::new();
        scenery_random::load_random(
            &mut game,
            &scenery_random::RandomMapCfg {
                size: 2,
                seed: 0,
                ships: 2,
                universe_cfg,
            },
        );

        let game = Rc::new(RefCell::new(game));

        let state = State {
            game: game,
            screen: StateScreen::Galaxy,
            selected_sector: 0,
            selected_fleet: 0,
            sector_view_transform: Transform2::identity(),
            time_speed: TimeSpeed::Normal,
            selected_object: None,
        };

        state
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test1() {}
}
