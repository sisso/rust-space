use crate::game_api::GameApi;
use godot::engine::node::InternalMode;
use godot::engine::{Button, Engine, GridContainer};
use godot::prelude::*;
use space_domain::game::sectors::SectorId;
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
        if let Some(sector_id) = self.get_clicked_sector() {
            GameApi::get_instance(self.base.share())
                .bind_mut()
                .on_click_sector(sector_id);
        }
    }

    #[func]
    pub fn on_click_fleet(&mut self) {
        godot_print!("on click fleet received");
    }

    pub fn get_clicked_sector(&self) -> Option<SectorId> {
        let container = self.get_sectors_container();
        let children = container.get_children(false);
        for (i, node) in children.iter_shared().enumerate() {
            if let Some(button) = node.try_cast::<Button>() {
                if button.is_pressed() {
                    return self.buttons_sectors.get(i).copied();
                }
            }
        }

        None
    }

    pub fn show_sectors(&mut self, sectors: Vec<LabeledEntity>) {
        self.buttons_sectors.clear();

        let mut grid = self.get_sectors_container();
        crate::utils::clear(grid.share());

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
        let grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/SectorsGridContainer");
        grid
    }

    pub fn show_fleets(&self, fleets: Vec<String>) {
        let mut grid = self.get_fleets_group();
        crate::utils::clear(grid.share());
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
        let grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer");
        grid
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

    fn process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
