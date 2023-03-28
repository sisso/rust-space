use godot::engine::node::InternalMode;
use godot::engine::{BoxContainer, Button, Container, Engine, GridContainer, HBoxContainer};
use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl MainGui {
    #[func]
    pub fn on_click_sector(&mut self) {
        godot_print!("on click sector received");
    }

    #[func]
    pub fn on_click_fleet(&mut self) {
        godot_print!("on click fleet received");
    }

    pub fn show_sectors(&self, sectors: Vec<String>) {
        godot_print!("MainGui show_sectors");
        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/SectorsGridContainer");
        Self::clear(grid.share());

        let columns = (sectors.len() as f32).sqrt().floor() as i64;
        grid.set_columns(columns);

        for sector in sectors {
            let mut button = Button::new_alloc();
            button.set_text(sector.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.share(), "on_click_sector"),
                0,
            );
            grid.add_child(button.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        }
    }

    pub fn show_fleets(&self, fleets: Vec<String>) {
        godot_print!("MainGui show_fleets");

        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer");
        Self::clear(grid.share());
        grid.set_columns(1);

        for fleet in fleets {
            let mut button = Button::new_alloc();
            button.set_text(fleet.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.share(), "on_click_fleet"),
                0,
            );
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
impl Node2DVirtual for MainGui {
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
