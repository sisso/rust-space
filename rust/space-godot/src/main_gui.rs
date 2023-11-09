use crate::utils::clear;
use godot::engine::{
    Button, Container, Engine, GridContainer, ItemList, RichTextLabel, TabContainer, TextEdit,
    Texture2D,
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
    building_sites: Vec<Id>,
    selected_building_site: Option<Id>,
    selected_action: Option<Id>,
}

pub struct LabeledId {
    pub id: Id,
    pub label: String,
}

pub enum Description {
    None,
    Obj {
        title: String,
        desc: String,
        actions: Vec<LabeledId>,
    },
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

    #[func]
    pub fn on_click_plot(&mut self) {
        self.selected_building_site = self.get_plot_item_selected();
    }

    #[func]
    pub fn on_mouse_entered(&mut self) {}

    #[func]
    pub fn on_mouse_exited(&mut self) {}

    #[func]
    pub fn on_click_selected_action(&mut self) {
        let container = self.get_selected_actions_container();
        let children = container.get_children();
        for node in children.iter_shared() {
            if let Some(button) = node.try_cast::<Button>() {
                if button.is_pressed() {
                    let id: i64 = button.get_meta("id".into()).to();
                    let id = id as Id;
                    self.selected_action = Some(id);
                }
            }
        }
    }

    pub fn get_plot_item_selected(&self) -> Option<Id> {
        let mut list = self.get_plot_list();
        let idx_list = list.get_selected_items();
        if idx_list.len() == 0 {
            None
        } else {
            self.building_sites.get(idx_list.get(0) as usize).cloned()
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

    pub fn take_selected_building_site(&mut self) -> Option<Id> {
        // godot_print!("take selected sector {:?}", self.selected_fleet);
        self.selected_building_site.take()
    }

    pub fn take_selected_action(&mut self) -> Option<Id> {
        self.selected_action.take()
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

    fn get_console_input(&self) -> Gd<TextEdit> {
        let grid = self
            .base
            .get_node_as::<TextEdit>("TabContainer/Console/Input");
        grid
    }

    fn get_console_output(&self) -> Gd<TextEdit> {
        let grid = self
            .base
            .get_node_as::<TextEdit>("TabContainer/Console/Input");
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
                "pressed".into(),
                Callable::from_object_method(self.base.clone(), "on_click_fleet"),
            );
            grid.add_child(button.upcast());
            self.buttons_fleets.push(fleet.id);
        }
    }

    fn get_selected_actions_container(&self) -> Gd<Container> {
        self.base
            .get_node_as::<Container>("TabContainer/Main/SelectedActions")
    }

    pub fn show_selected_plot_desc(&mut self, desc: Description) {
        let mut rich_text = self.base.get_node_as::<RichTextLabel>(
            "TabContainer/Construction/VBoxContainer/PlotDescriptionRichTextLabel",
        );

        let text = match desc {
            Description::None => "none".to_string(),
            Description::Obj {
                mut title, desc, ..
            } => {
                title.push('\n');
                title.push_str(&desc);
                title
            }
        };
        rich_text.set_text(text.into());
    }

    pub fn show_buildings_sites(&mut self, buildings: Vec<LabeledId>) {
        let mut list = self.get_plot_list();
        let selected_idx = {
            let select_items = list.get_selected_items();
            if select_items.len() == 0 {
                0
            } else {
                select_items.get(0)
            }
        };
        list.clear();
        self.building_sites.clear();
        let list_len = buildings.len();
        for i in buildings {
            // hack https://github.com/godot-rust/gdext/issues/391
            // list.add_item(building.label.into())
            let idx = list.add_icon_item(Texture2D::new());
            list.set_item_text(idx, i.label.into());
            self.building_sites.push(i.id);
        }
        if selected_idx < list_len as i32 {
            list.select(selected_idx);
        } else {
            list.select(0);
        }
    }

    fn get_fleets_container(&self) -> Gd<GridContainer> {
        self.base
            .get_node_as::<GridContainer>("TabContainer/Main/FleetsGridContainer")
    }

    fn get_plot_list(&self) -> Gd<ItemList> {
        self.base
            .get_node_as::<ItemList>("TabContainer/Construction/VBoxContainer/PlotItemList")
    }

    fn get_plot_button(&self) -> Gd<Button> {
        self.base
            .get_node_as::<Button>("TabContainer/Construction/VBoxContainer/PlotButton")
    }

    fn get_main_container(&self) -> Gd<TabContainer> {
        self.base.get_node_as::<TabContainer>("TabContainer")
    }

    pub fn show_selected_object(&mut self, desc: Description) {
        // update rich text
        let mut rich_text = self
            .base
            .get_node_as::<RichTextLabel>("TabContainer/Main/SelectedObjRichTextLabel");

        let (text, actions) = match desc {
            Description::None => ("none".to_string(), vec![]),
            Description::Obj {
                mut title,
                desc,
                actions,
            } => {
                title.push('\n');
                title.push_str(&desc);
                (title, actions)
            }
        };
        rich_text.set_text(text.into());

        self.update_selected_actions(actions);
    }

    fn update_selected_actions(&mut self, actions: Vec<LabeledId>) {
        let mut container = self.get_selected_actions_container();
        if container.get_child_count() as usize == actions.len() {
            return;
        }

        clear(container.clone());

        for LabeledId { id, label } in actions {
            let mut button = Button::new_alloc();
            button.set_text(label.into());
            button.connect(
                "button_down".into(),
                Callable::from_object_method(self.base.clone(), "on_click_selected_action"),
            );
            button.set_meta("id".into(), Variant::from(id as i64));
            container.add_child(button.upcast());
        }
    }
}

#[godot_api]
impl Node2DVirtual for MainGui {
    fn init(base: Base<Node2D>) -> Self {
        let gui = Self {
            base,
            buttons_sectors: vec![],
            buttons_fleets: vec![],
            building_sites: vec![],
            selected_sector: None,
            selected_fleet: None,
            selected_building_site: None,
            selected_action: None,
        };

        if Engine::singleton().is_editor_hint() {
        } else {
        }

        gui
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
            // register handlers
            let mut plot_button = self.get_plot_button();
            plot_button.connect(
                "pressed".into(),
                Callable::from_object_method(self.base.clone(), "on_click_plot"),
            );

            let mut tab_container = self.get_main_container();
            tab_container.connect(
                "mouse_entered".into(),
                Callable::from_object_method(self.base.clone(), "on_mouse_entered"),
            );
            tab_container.connect(
                "mouse_exited".into(),
                Callable::from_object_method(self.base.clone(), "on_mouse_exited"),
            );
        }
    }

    fn process(&mut self, _delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
