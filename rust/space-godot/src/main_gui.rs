use godot::engine::node::InternalMode;
use godot::engine::{BoxContainer, Button, Container, Engine, GridContainer, HBoxContainer};
use godot::prelude::*;
use specs::Entity;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
    buttons_sectors: Vec<Entity>,
    buttons_fleets: Vec<Entity>,
}

pub struct LabeledEntity {
    pub id: Entity,
    pub label: String,
}

#[godot_api]
impl MainGui {
    #[func]
    pub fn on_click_sector(&mut self) {
        godot_print!("on click sector received");
        let container = self.get_sectors_container();
        let children = container.get_children(false);
        for (i, node) in children.iter_shared().enumerate() {
            if let Some(button) = node.try_cast::<Button>() {
                if button.is_pressed() {
                    godot_print!("clicked on {:?}", self.buttons_sectors.get(i));
                }
            }
        }
    }

    #[func]
    pub fn on_click_fleet(&mut self) {
        godot_print!("on click fleet received");
    }

    pub fn show_sectors(&mut self, sectors: Vec<LabeledEntity>) {
        godot_print!("MainGui show_sectors");
        self.buttons_sectors.clear();

        let mut grid = self.get_sectors_container();
        Self::clear(grid.share());

        let columns = (sectors.len() as f32).sqrt().floor() as i64;
        grid.set_columns(columns);

        for le in sectors {
            let mut button = Button::new_alloc();
            button.set_text(le.label.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.share(), "on_click_sector"),
                0,
            );
            grid.add_child(button.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
            self.buttons_sectors.push(le.id);
        }
    }

    fn get_sectors_container(&self) -> Gd<GridContainer> {
        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/SectorsGridContainer");
        grid
    }

    pub fn show_fleets(&self, fleets: Vec<String>) {
        godot_print!("MainGui show_fleets");

        let mut grid = self.get_fleets_group();
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

    fn get_fleets_group(&self) -> Gd<GridContainer> {
        let mut grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer");
        grid
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

        Self {
            base,
            buttons_sectors: vec![],
            buttons_fleets: vec![],
        }
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
