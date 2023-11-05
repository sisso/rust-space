use godot::log::{godot_print, godot_warn};
use godot::obj::Gd;

use space_flap::{Id, ObjAction, ObjActionKind, ObjCargo, ObjData, ObjDesc, WareData};

use crate::main_gui::{Description, LabeledId, MainGui};
use crate::sector_view::SectorView;
use crate::state::{State, StateScreen};
use crate::{main_gui, sector_view};

pub struct Runtime {
    state: State,
    sector_view: Gd<SectorView>,
    gui: Gd<MainGui>,
}

impl Runtime {
    pub fn new(state: State, sector_view: Gd<SectorView>, gui: Gd<MainGui>) -> Self {
        Self {
            state,
            sector_view,
            gui,
        }
    }

    pub fn tick(&mut self, delta_seconds: f64) {
        // process inputs
        let sector_selected_id = self.sector_view.bind_mut().take_selected_id();
        let building_plot_position = self.sector_view.bind_mut().take_build_plot();
        let clicked_sector_id = self.gui.bind_mut().take_selected_sector_id();
        let clicked_fleet_id = self.gui.bind_mut().take_selected_fleet_id();
        let clicked_to_build_plot_id = self.gui.bind_mut().take_selected_building_site();
        let clicked_selected_action = self.gui.bind_mut().take_selected_action();

        // check each input generated by subsystem, GUI inputs must be handled before "sector_view"
        // inputs that can not identify if the click was over a GUI element.
        if let Some(sector_id) = clicked_sector_id {
            // when click on sector, clear any selected element and move to the sector
            // clear selected element
            self.sector_view
                .bind_mut()
                .set_state(sector_view::SectorViewState::None);
            self.state.screen = StateScreen::Sector(sector_id);
        } else if let Some(fleet_id) = clicked_fleet_id {
            // when click on a fleet, start to follow that fleet

            // sometimes fleet is docked, we didn't handle it properly (I think)
            self.sector_view
                .bind_mut()
                .set_state(sector_view::SectorViewState::None);
            self.state.screen = StateScreen::Obj(fleet_id);
        } else if let Some(prefab_id) = clicked_selected_action {
            let r = self.set_shipyard_production(prefab_id);
            godot_print!("set shipyard prefab_id {:?}: {:?}", prefab_id, r);
        } else if let Some(id) = clicked_to_build_plot_id {
            // user want to build a plot
            let current_sector_id = self.state.get_current_sector_id();
            self.state.screen = StateScreen::SectorPlot {
                sector_id: current_sector_id,
                plot_id: id,
            };
            self.sector_view
                .bind_mut()
                .set_state(sector_view::SectorViewState::Plotting);
        } else if let Some(pos) = building_plot_position {
            // user select place to plot
            match self.state.screen {
                StateScreen::SectorPlot { plot_id, sector_id } => {
                    self.sector_view
                        .bind_mut()
                        .set_state(sector_view::SectorViewState::None);
                    self.state.screen = StateScreen::Sector(sector_id);

                    let id = self
                        .state
                        .game
                        .new_building_plot(plot_id, sector_id, pos.x, pos.y);
                    godot_print!(
                        "added building plot {:?} at {:?}/{:?}: {:?}",
                        plot_id,
                        sector_id,
                        pos,
                        id
                    );
                }
                _ => {
                    godot_warn!("plot position selected by game_api is not on building plot state");
                }
            }
        } else if let Some(id) = sector_selected_id {
            // when has on a obj in the sector already selected
            self.state.screen = StateScreen::Obj(id);
        }

        // update game
        self.state.game.update(delta_seconds as f32);

        // update view
        self.refresh_sector_view();

        // take events and check if new we have changes on sector | fleets | prefabs list and do a
        // full ui refresh
        // let events = self.state.game.take_events();
        // self.refresh_gui();
        // or else, do only realtime ui updates
        self.refresh_gui();
    }

    pub fn refresh_sector_view(&mut self) {
        match &self.state.screen {
            StateScreen::Sector(sector_id) => {
                self.sector_view
                    .bind_mut()
                    .refresh(generate_sectorview_updates(&self.state, *sector_id));
                self.gui
                    .bind_mut()
                    .show_selected_object(main_gui::Description::None);
            }
            StateScreen::Obj(id) => {
                let sector_id = self
                    .state
                    .game
                    .get_obj_coords(*id)
                    .map(|coords| coords.get_sector_id());

                if let Some(sector_id) = sector_id {
                    self.sector_view
                        .bind_mut()
                        .refresh(generate_sectorview_updates(&self.state, sector_id));
                }
                let desc = self.describe_obj(*id);
                self.gui.bind_mut().show_selected_object(desc);
            }

            StateScreen::SectorPlot { sector_id, plot_id } => {
                let updates = generate_sectorview_updates(&self.state, *sector_id);
                self.sector_view.bind_mut().refresh(updates);

                let describe_plot = describe_plot(&self.state.game, *plot_id);

                let mut gui = self.gui.bind_mut();
                gui.show_selected_plot_desc(describe_plot);
                gui.show_selected_object(main_gui::Description::None);
            }
        }
    }

    /// update gui, it happens every tick
    pub fn refresh_gui(&mut self) {
        let describe_plot = if let Some(plot_id) = self.gui.bind().get_plot_item_selected() {
            describe_plot(&self.state.game, plot_id)
        } else {
            godot_print!("no plot selected????");
            Description::None
        };

        let mut gui = self.gui.bind_mut();
        gui.show_selected_plot_desc(describe_plot);
    }

    /// full refresh gui, should happens when things happens, like new fleet is added / removed
    pub fn full_refresh_gui(&mut self) {
        let mut sectors = vec![];
        for sector in self.state.game.get_sectors() {
            sectors.push(LabeledId {
                id: sector.get_id(),
                label: sector.get_label().to_string(),
            })
        }

        let mut fleets = vec![];
        for fleet in self.state.game.get_fleets() {
            fleets.push(LabeledId {
                id: fleet.get_id(),
                label: format!(
                    "{} ({})",
                    self.state
                        .game
                        .get_label(fleet.get_id())
                        .expect("fleet has no label"),
                    fleet.get_id()
                ),
            })
        }

        let mut buildings = vec![];
        for pf in self.state.game.list_building_sites_prefabs() {
            godot_print!("- {:?} {:?}", pf.get_id(), pf.get_label());
            buildings.push(LabeledId {
                id: pf.get_id(),
                label: format!("BS {}", pf.get_label()),
            })
        }

        let mut gui = self.gui.bind_mut();
        gui.show_sectors(sectors);
        gui.show_fleets(fleets);
        gui.show_buildings_sites(buildings);
    }

    pub fn recenter(&mut self) {
        self.sector_view.bind_mut().recenter();
    }

    pub fn describe_obj(&self, id: Id) -> Description {
        let data = self.state.game.get_obj(id);
        let desc = self.state.game.get_obj_desc(id);
        let jump_target = self
            .state
            .game
            .get_jump(id)
            .and_then(|jump| self.state.game.get_obj_desc(jump.get_to_sector_id()))
            .map(|target_desc| target_desc.get_label().to_string());

        let docked_fleets = desc
            .as_ref()
            .map(|desc| desc.get_docked_fleets())
            .map(|docked_fleets_id| {
                docked_fleets_id
                    .into_iter()
                    .map(|id| {
                        self.state
                            .game
                            .get_label(id)
                            .expect("docked obj has no label")
                    })
                    .collect()
            })
            .unwrap_or(vec![]);

        let mut actions = vec![];

        if data.is_none() || desc.is_none() {
            return Description::None;
        }

        let data = data.unwrap();
        let desc = desc.unwrap();

        let kind = get_kind_str(&data);
        let mut buffer = vec![format!("{} {:?}", kind, data.get_id())];
        if let Some(action) = desc.get_action() {
            buffer.push(get_action_string(action));
        }
        if let Some(target_id) = desc.get_nav_move_to_target() {
            buffer.push(format!("target id: {:?}", target_id));
        }
        if let Some(cargo) = desc.get_cargo() {
            buffer.extend(get_cargo_str(&self.state.wares, cargo));
        }
        if let Some(factory) = desc.get_factory() {
            if factory.is_producing() {
                buffer.push("producing".to_string());
            }
        }
        if let Some(shipyard) = desc.get_shipyard() {
            let prefabs = self.state.game.list_building_shipyard_prefabs();

            if let Some(prefab_id) = shipyard.get_order() {
                let prefab = prefabs
                    .iter()
                    .find(|prefab_data| prefab_data.get_id() == prefab_id)
                    .expect("order prefab not found");

                buffer.push(format!("current order: {:?}", prefab.get_label()));
            }

            if let Some(prefab_id) = shipyard.get_producing_prefab_id() {
                let prefab = prefabs
                    .iter()
                    .find(|prefab_data| prefab_data.get_id() == prefab_id)
                    .expect("producing prefab not found");
                buffer.push(format!("producing {}", prefab.get_label()));
            }

            actions.extend(prefabs.into_iter().map(|prefab| LabeledId {
                id: prefab.get_id(),
                label: prefab.get_label().to_string(),
            }));
        }
        if let Some(target_sector) = jump_target {
            buffer.push(format!("jump to {}", target_sector));
        }

        if docked_fleets.len() > 0 {
            buffer.push("docked:".to_string());
            for fleet in docked_fleets {
                buffer.push(format!("- {}", fleet));
            }
        }

        let orders = data.get_trade_orders();
        if !orders.is_empty() {
            buffer.push("trade orders:".to_string());
            for order in orders {
                let order_kind = if order.is_provide() {
                    "provide"
                } else {
                    "request"
                };

                let ware_name = get_ware_label(&self.state.wares, order.get_ware());
                buffer.push(format!("{}: {}", order_kind, ware_name));
            }
        }

        Description::Obj {
            title: desc.get_label().to_string(),
            desc: buffer.join("\n"),
            actions,
        }
    }

    fn set_shipyard_production(&mut self, prefab_id: Id) -> Option<()> {
        // double check selected entity is a shipyard
        godot_print!("get state screen");
        let obj_id = match &self.state.screen {
            StateScreen::Obj(id) => *id,
            _ => return None,
        };
        godot_print!("get obj desc");
        let dsc = self.state.game.get_obj_desc(obj_id)?;
        godot_print!("get obj desc shipyard");
        let _ = dsc.get_shipyard()?;

        // add order
        godot_print!("adding order");
        let added = self
            .state
            .game
            .add_shipyard_building_order(obj_id, prefab_id);
        godot_print!("order result {:?}", added);
        Some(())
    }
}

pub fn describe_plot(game: &space_flap::SpaceGame, id: Id) -> main_gui::Description {
    let ds = game.get_obj_desc(id);
    if ds.is_none() {
        godot_warn!("description for plot {:?} not found", id);
        return Description::None;
    }

    let ds = ds.unwrap();

    Description::Obj {
        title: ds.get_label().into(),
        desc: "Building plot".to_string(),
        actions: vec![],
    }
}

fn get_cargo_str(wares: &Vec<WareData>, cargo: ObjCargo) -> Vec<String> {
    let mut b = vec![];
    b.push("Cargo:".to_string());
    if b.is_empty() {
        b.push("<empty>".to_string());
    } else {
        for (id, amount) in cargo.get_wares() {
            let ware_label = get_ware_label(wares, id);
            b.push(format!("- {}: {}", ware_label, amount))
        }
    }
    b
}

fn get_ware_label(wares: &Vec<WareData>, id: Id) -> String {
    wares
        .iter()
        .find(|i| i.get_id() == id)
        .map(|l| l.get_label().to_string())
        .unwrap_or_else(|| format!("id {}", id))
}

fn get_action_string(action: ObjAction) -> String {
    match action.get_kind() {
        ObjActionKind::Undock => "undock".to_string(),
        ObjActionKind::Jump => "jump".to_string(),
        ObjActionKind::Dock => "dock".to_string(),
        ObjActionKind::MoveTo => "move to".to_string(),
        ObjActionKind::MoveToTargetPos => "move to target".to_string(),
        ObjActionKind::Extract => "extract".to_string(),
    }
}

fn get_kind_str(data: &ObjData) -> &str {
    if data.is_jump() {
        "jump"
    } else if data.is_astro() {
        "planet"
    } else if data.is_fleet() {
        "fleet"
    } else if data.is_station() {
        if data.is_shipyard() {
            "shipyard"
        } else if data.is_factory() {
            "factory station"
        } else {
            "station"
        }
    } else if data.is_jump() {
        "jump"
    } else if data.is_astro_star() {
        "star"
    } else if data.is_asteroid() {
        "asteroid"
    } else {
        "unknown"
    }
}

pub fn generate_sectorview_updates(state: &State, sector_id: Id) -> Vec<sector_view::Update> {
    let mut updates = vec![];

    let list = state
        .game
        .list_at_sector(sector_id)
        .into_iter()
        .flat_map(|id| state.game.get_obj(id));

    for data in list {
        updates.push(sector_view::Update::Obj {
            id: data.get_id(),
            pos: data.get_coords().into(),
            kind: sector_view::ObjKind {
                fleet: data.is_fleet(),
                jump: data.is_jump(),
                station: data.is_station(),
                asteroid: data.is_asteroid(),
                astro: data.is_astro(),
                astro_star: data.is_astro_star(),
            },
        });

        if let Some(orbit) = data.get_orbit() {
            updates.push(sector_view::Update::Orbit {
                id: data.get_id(),
                pos: data.get_coords().into(),
                parent_pos: orbit.get_parent_pos().into(),
                radius: orbit.get_radius(),
            })
        }
    }

    updates
}
