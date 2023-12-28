use godot::bind::{godot_api, GodotClass};
use godot::engine::{Engine, INode, Node, NodeExt};
use godot::obj::Base;
use godot::prelude::*;

use crate::game_api_runtime::Runtime;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    runtime: Option<Runtime>,
    #[export(enum = (Warn = 0, Info = 1, Debug = 2 , Trace = 3))]
    debug_level: i32,
    #[base]
    base: Base<Node>,
}

fn resolve_log_i32(level: i32) -> log::LevelFilter {
    match level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    }
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

    // pub fn on_click_sector(&mut self, sector_id: Id) {
    //     let runtime = self.runtime.as_mut().expect("runtime not initialized");
    //     runtime.change_sector(sector_id);
    // }

    // pub fn on_click_fleet(&mut self, fleet_id: Id) {
    //     let runtime = self.runtime.as_mut().expect("runtime not initialized");
    //     runtime.on_selected_entity(Some(fleet_id));
    // }

    // pub fn on_selected_entity(&mut self, id: Option<Id>) {
    //     godot_print!("on selected on sector {:?}", id);
    //     let runtime = self.runtime.as_mut().expect("runtime not initialized");
    //     runtime.on_selected_entity(id);
    // }
}

#[godot_api]
impl INode for GameApi {
    fn init(base: Base<Node>) -> Self {
        if Engine::singleton().is_editor_hint() {
            GameApi {
                runtime: None,
                base: base,
                debug_level: 2,
            }
        } else {
            GameApi {
                runtime: None,
                base: base,
                debug_level: 2,
            }
        }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
            let log_level = resolve_log_i32(self.debug_level);
            godot_print!("using debug level {:?}", log_level);

            let sector_view = self
                .base
                .try_get_node_as("/root/GameApi/SectorView")
                .expect("SectorView not found");
            let gui = self
                .base
                .try_get_node_as("/root/GameApi/MainGui")
                .expect("MainGui not found");

            let mut runtime = Runtime::new(sector_view, gui, log_level);
            runtime.full_refresh_gui();
            runtime.recenter();
            runtime.refresh_sector_view();

            self.runtime = Some(runtime);
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        let runtime = self.runtime.as_mut().expect("runtime not initialized");
        runtime.tick(delta);
    }
}
