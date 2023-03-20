use commons::math::{Transform2, P2, V2};
use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;
use space_domain::game::{scenery_random, Game};
use specs::Entity;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct SpaceGame;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceGame {}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct GameApi {
    state: State,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl GameApi {}

#[godot_api]
impl GodotExt for GameApi {
    fn init(base: Base<Node2D>) -> Self {
        let state = State::new();
        godot_print!("init");
        GameApi { state, base: base }
    }

    fn ready(&mut self) {
        godot_print!("ready");
    }

    fn process(&mut self, delta: f64) {
        // godot_print!("update for {delta}");
        // self.translate(Vector2::new(delta as f32, 0.0));
    }
}

enum StateScreen {
    Sector(Entity),
    Galaxy,
    Fleet(Entity),
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum TimeSpeed {
    Pause,
    Normal,
}

pub struct State {
    game: Rc<RefCell<Game>>,
    screen: StateScreen,
    selected_sector: usize,
    selected_fleet: usize,
    selected_object: Option<Entity>,
    sector_view_transform: Transform2,
    time_speed: TimeSpeed,
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
                size: 4,
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
    use super::*;

    #[test]
    fn test1() {}
}
