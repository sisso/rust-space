use godot::engine::node::InternalMode;
use godot::engine::{BoxContainer, Button, Container, Engine, GridContainer, HBoxContainer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl MainGui {
    #[func]
    pub fn show_sectors(&self) {
        godot_print!("MainGui show_sectors");
        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/SectorsGridContainer");
        Self::clear(grid.share());

        grid.set_columns(2);

        for i in 0..4 {
            let mut button = Button::new_alloc();
            button.set_text(format!("0 {i}").into());
            grid.add_child(button.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        }
    }

    #[func]
    pub fn show_fleets(&self) {
        godot_print!("MainGui show_fleets");

        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer");
        Self::clear(grid.share());
        grid.set_columns(1);

        for i in 0..4 {
            let mut button = Button::new_alloc();
            button.set_text(format!("Fleet {i}").into());
            grid.add_child(button.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        }
    }

    fn clear<T>(container: Gd<T>)
    where
        T: Inherits<Container>,
    {
        let mut container = container.upcast();

        for c in container.get_children(true).iter_shared() {
            let mut n = c.cast::<Node>();
            container.remove_child(n.share());
            n.queue_free();
        }
    }
}

#[godot_api]
impl GodotExt for MainGui {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        Self { base }
    }

    fn ready(&mut self) {
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
