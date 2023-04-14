use godot::bind::{godot_api, GodotClass};
use godot::engine::{Engine, Node, NodeExt, NodeVirtual};
use godot::obj::Base;
use godot::prelude::*;

use space_flap::*;

use crate::runtime::Runtime;
use crate::state::State;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    runtime: Option<Runtime>,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl GameApi {
    // pub fn get_instance<T>(provided: Gd<T>) -> Gd<GameApi>
    // where
    //     T: Inherits<Node>,
    // {
    //     let node = provided.upcast();
    //     node.get_node_as::<GameApi>("/root/GameApi")
    // }

    pub fn on_click_sector(&mut self, sector_id: Id) {
        let runtime = self.runtime.as_mut().expect("runtime not initialized");
        runtime.change_sector(sector_id);
    }

    pub fn on_selected_entity(&mut self, id: Option<Id>) {
        godot_print!("on selected on sector {:?}", id);
        let runtime = self.runtime.as_mut().expect("runtime not initialized");
        runtime.on_selected_entity(id);
    }
}

#[godot_api]
impl NodeVirtual for GameApi {
    fn init(base: Base<Node>) -> Self {
        if Engine::singleton().is_editor_hint() {
            GameApi {
                runtime: None,
                base: base,
            }
        } else {
            GameApi {
                runtime: None,
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
            let sector_view = self
                .try_get_node_as("/root/GameApi/SectorView")
                .expect("SectorView not found");
            let gui = self
                .try_get_node_as("/root/GameApi/MainGui")
                .expect("MainGui not found");

            let state = State::new();

            let mut runtime = Runtime::new(state, sector_view, gui);
            runtime.update_gui();
            runtime.recenter();
            runtime.refresh_sector_view();

            self.runtime = Some(runtime);
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        let runtime = self.runtime.as_mut().expect("runtime not intiialized");
        runtime.tick(delta);
    }
}
