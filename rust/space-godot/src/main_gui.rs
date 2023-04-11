use crate::game_api::GameApi;
use godot::engine::node::InternalMode;
use godot::engine::{Button, Engine, GridContainer, RichTextLabel, TabContainer};
use godot::prelude::*;
use space_flap::Id;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
    buttons_sectors: Vec<Id>,
    buttons_fleets: Vec<Id>,
}

pub struct LabeledId {
    pub id: Id,
    pub label: String,
}

pub enum Description {
    None,
    Obj { title: String, desc: String },
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

    pub fn get_clicked_sector(&self) -> Option<Id> {
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

    pub fn show_sectors(&mut self, sectors: Vec<LabeledId>) {
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

    pub fn show_fleets(&self, fleets: Vec<LabeledId>) {
        let mut grid = self.get_fleets_group();
        crate::utils::clear(grid.share());
        grid.set_columns(1);

        for fleet in fleets {
            let mut button = Button::new_alloc();
            button.set_text(fleet.label.into());
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

    pub fn show_selected_object(&mut self, d: Description) {
        // update rich text
        let mut rich_text = self
            .base
            .get_node_as::<RichTextLabel>("TabContainer/Details/SelectedObjRichTextLabel");

        let text = match d {
            Description::None => "none".to_string(),
            Description::Obj { mut title, desc } => {
                title.push('\n');
                title.push_str(&desc);
                title
            }
        };
        rich_text.set_text(text.into());

        // automatic swich to tab to details
        let mut tabs = self.base.get_node_as::<TabContainer>("TabContainer");
        if tabs.get_current_tab() != 1 {
            tabs.set_current_tab(1);
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

    fn process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
