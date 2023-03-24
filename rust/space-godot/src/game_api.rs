use crate::main_gui::MainGui;
use crate::state::State;
use commons::math::Transform2;
use godot::bind::{godot_api, GodotClass, GodotExt};
use godot::builtin::GodotString;
use godot::engine::{Engine, Node, NodeExt};
use godot::log::godot_print;
use godot::obj::Base;
use space_domain::game::fleets::Fleet;
use space_domain::game::sectors::Sector;
use space_domain::game::{scenery_random, Game};
use specs::{Entity, WorldExt};
use std::cell::{Ref, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    state: Option<State>,
    #[base]
    base: Base<Node>,
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

    pub fn update_gui(&mut self) {
        let state = self.get_state();

        godot_print!("GameApi.update_gui");
        let gui = self
            .base
            .get_parent()
            .unwrap()
            .find_child("MainGui".into(), true, false);
        let mut gui = gui.unwrap().cast::<MainGui>();
        let gui = gui.bind_mut();

        let sectors_storage = state.world.read_storage::<Sector>();
        let sectors: Vec<String> = sectors_storage
            .as_slice()
            .iter()
            .map(|i| format!("{} {}", i.coords.x, i.coords.y))
            .collect();
        gui.show_sectors(sectors);

        let fleets_storage = state.world.read_storage::<Fleet>();
        let fleets: Vec<String> = fleets_storage
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, fleet)| format!("Fleet {}", i))
            .collect();
        gui.show_fleets(fleets);

        godot_print!("GameApi.update_gui end");
    }
}

impl GameApi {
    fn get_state(&self) -> Ref<'_, Game> {
        self.state
            .as_ref()
            .expect("state not initialized")
            .game
            .borrow()
    }
}

#[godot_api]
impl GodotExt for GameApi {
    fn init(base: Base<Node>) -> Self {
        if Engine::singleton().is_editor_hint() {
            GameApi {
                state: None,
                // value: 1,
                // value_str: Default::default(),
                base: base,
            }
        } else {
            let state = State::new();
            GameApi {
                state: Some(state),
                // value: 2,
                // value_str: Default::default(),
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        // godot_print!("ready {} {}", self.value, self.value_str);
        godot_print!("ready 2");
        if Engine::singleton().is_editor_hint() {
        } else {
            godot_print!("ready self.update_gui");
            self.update_gui();
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
