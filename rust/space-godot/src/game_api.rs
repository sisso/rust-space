use crate::main_gui::MainGui;
use crate::state::State;
use commons::math::Transform2;
use godot::bind::{godot_api, GodotClass, GodotExt};
use godot::builtin::GodotString;
use godot::engine::{Engine, Node, NodeExt};
use godot::log::godot_print;
use godot::obj::Base;
use space_domain::game::{scenery_random, Game};
use specs::Entity;
use std::cell::RefCell;
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
        godot_print!("GameApi.update_gui");
        let gui = self
            .base
            .get_parent()
            .unwrap()
            .find_child("MainGui".into(), true, false);
        let mut gui = gui.unwrap().cast::<MainGui>();
        let gui = gui.bind_mut();
        gui.show_sectors();
        gui.show_fleets();
        godot_print!("GameApi.update_gui end");
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
