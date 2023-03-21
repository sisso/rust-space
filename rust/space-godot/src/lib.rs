use commons::math::{Transform2, P2, V2};
use godot::engine::Engine;
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
    state: Option<State>,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl GameApi {
    #[func]
    pub fn add(&self, a: i64, b: i64) -> i64 {
        a + b
    }

    #[func]
    pub fn get_u64(&self) -> i64 {
        1
    }

    #[func]
    pub fn get_f32(&self) -> f32 {
        2.33
    }

    #[func]
    pub fn get_string(&self) -> GodotString {
        "one".into()
    }
}

#[godot_api]
impl GodotExt for GameApi {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
            godot_print!("init empty");
            GameApi {
                state: None,
                base: base,
            }
        } else {
            let state = State::new();
            godot_print!("init state");
            GameApi {
                state: Some(state),
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        godot_print!("ready");
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
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
