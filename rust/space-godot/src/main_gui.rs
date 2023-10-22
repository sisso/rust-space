use godot::engine::{
    Button, Engine, GridContainer, ItemList, RichTextLabel, Texture2D, VBoxContainer,
};
use godot::prelude::*;
use space_flap::Id;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
    buttons_sectors: Vec<Id>,
    buttons_fleets: Vec<Id>,
    selected_sector: Option<Id>,
    selected_fleet: Option<Id>,
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
            self.selected_sector = Some(sector_id);
        }
    }

    #[func]
    pub fn on_click_fleet(&mut self) {
        if let Some(fleet_id) = self.get_clicked_fleet() {
            self.selected_fleet = Some(fleet_id);
        }
    }

    pub fn get_clicked_fleet(&mut self) -> Option<Id> {
        let container = self.get_fleets_container();
        let children = container.get_children();
        for (i, node) in children.iter_shared().enumerate() {
            if let Some(button) = node.try_cast::<Button>() {
                if button.is_pressed() {
                    return self.buttons_fleets.get(i).copied();
                }
            }
        }

        None
    }

    pub fn take_selected_sector_id(&mut self) -> Option<Id> {
        // godot_print!("take selected sector {:?}", self.selected_sector);
        self.selected_sector.take()
    }

    pub fn take_selected_fleet_id(&mut self) -> Option<Id> {
        // godot_print!("take selected sector {:?}", self.selected_fleet);
        self.selected_fleet.take()
    }

    pub fn get_clicked_sector(&self) -> Option<Id> {
        let container = self.get_sectors_container();
        let children = container.get_children();
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
        crate::utils::clear(grid.clone());

        let columns = (sectors.len() as f32).sqrt().floor() as i32;
        grid.set_columns(columns);

        for le in sectors {
            let mut button = Button::new_alloc();
            button.set_text(le.label.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.clone(), "on_click_sector"),
            );
            grid.add_child(button.upcast());
            self.buttons_sectors.push(le.id);
        }
    }

    fn get_sectors_container(&self) -> Gd<GridContainer> {
        let grid = self
            .base
            .get_node_as::<GridContainer>("TabContainer/Main/SectorsGridContainer");
        grid
    }

    pub fn show_fleets(&mut self, fleets: Vec<LabeledId>) {
        let mut grid = self.get_fleets_container();
        crate::utils::clear(grid.clone());
        grid.set_columns(1);

        for fleet in fleets {
            let mut button = Button::new_alloc();
            button.set_text(fleet.label.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.clone(), "on_click_fleet"),
            );
            grid.add_child(button.upcast());
            self.buttons_fleets.push(fleet.id);
        }
    }

    pub fn show_buildings(&mut self, builgins: Vec<LabeledId>) {
        let mut list = self.get_buildings_container();
        list.clear();
        for building in builgins {
            // hack https://github.com/godot-rust/gdext/issues/391
            // list.add_item(building.label.into())
            let idx = list.add_icon_item(Texture2D::new());
            list.set_item_text(idx, building.label.into());
        }
    }

    fn get_fleets_container(&self) -> Gd<GridContainer> {
        self.base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer")
    }

    fn get_buildings_container(&self) -> Gd<ItemList> {
        self.base
            .get_node_as::<ItemList>("TabContainer/Construction/VBoxContainer/ItemList")
    }

    pub fn show_selected_object(&mut self, desc: Description) {
        // update rich text
        let mut rich_text = self
            .base
            .get_node_as::<RichTextLabel>("TabContainer/Main/SelectedObjRichTextLabel");

        let text = match desc {
            Description::None => "none".to_string(),
            Description::Obj { mut title, desc } => {
                title.push('\n');
                title.push_str(&desc);
                title
            }
        };
        rich_text.set_text(text.into());
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
            selected_sector: None,
            selected_fleet: None,
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
