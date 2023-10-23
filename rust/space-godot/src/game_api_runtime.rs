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
        let sector_selected_id = self.sector_view.bind().get_selected_id();
        let clicked_sector_id = self.gui.bind_mut().take_selected_sector_id();
        let clicked_fleet_id = self.gui.bind_mut().take_selected_fleet_id();
        let clicked_plot_id = self.gui.bind_mut().take_selected_building_site();

        if let Some(sector_id) = clicked_sector_id {
            // when click on sector, clear any selected element and move to the sector
            // clear selected element
            self.sector_view.bind_mut().set_selected(None);
            self.state.screen = StateScreen::Sector(sector_id);
        } else if let Some(fleet_id) = clicked_fleet_id {
            // when click on a fleet, start to follow that fleet

            // sometimes fleet is docked, we didn't handle it properly (I think)
            self.sector_view.bind_mut().set_selected(None);
            self.state.screen = StateScreen::Obj(fleet_id);
        } else if let Some(id) = clicked_plot_id {
            let current_sector_id = self.state.get_current_sector_id();

            self.sector_view.bind_mut().set_selected(None);
            self.state.screen = StateScreen::SectorPlot {
                sector_id: current_sector_id,
                plot_id: id,
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
        self.tick_refresh_gui();
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
                let mut params = generate_sectorview_updates(&self.state, *sector_id);
                params.building_plot = true;

                self.sector_view.bind_mut().refresh(params);

                let describe_plot = describe_plot(&self.state.game, *plot_id);

                let mut gui = self.gui.bind_mut();
                gui.show_selected_plot_desc(describe_plot);
                gui.show_selected_object(main_gui::Description::None);
            }
        }
    }

    pub fn tick_refresh_gui(&mut self) {
        let describe_plot = if let Some(plot_id) = self.gui.bind().get_plot_item_selected() {
            describe_plot(&self.state.game, plot_id)
        } else {
            godot_print!("no plot selected????");
            Description::None
        };

        let mut gui = self.gui.bind_mut();
        gui.show_selected_plot_desc(describe_plot);
    }

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
                label: format!("Fleet {}", fleet.get_id()),
            })
        }

        let mut buildings = vec![];
        for pf in self.state.game.list_building_sites_prefabs() {
            godot_print!("- {:?}", pf.get_label());
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

    // pub fn on_selected_entity(&mut self, id: Option<Id>) {
    //     self.state.selected_object = id;
    //
    //     if let Some(id) = self.state.selected_object {
    //         let uidesc = self.describe_obj(id);
    //         self.gui.bind_mut().show_selected_object(uidesc);
    //     } else {
    //         self.gui
    //             .bind_mut()
    //             .show_selected_object(main_gui::Description::None);
    //     }
    // }

    pub fn describe_obj(&self, id: Id) -> main_gui::Description {
        let dt = self.state.game.get_obj(id);
        let ds = self.state.game.get_obj_desc(id);
        let jump_target = self
            .state
            .game
            .get_jump(id)
            .and_then(|jump| self.state.game.get_obj_desc(jump.get_to_sector_id()))
            .map(|target_desc| target_desc.get_label().to_string());
        describe_obj(&self.state.wares, dt, ds, jump_target)
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
    }
}

fn describe_obj(
    wares: &Vec<WareData>,
    data: Option<ObjData>,
    desc: Option<ObjDesc>,
    jump_target_sector: Option<String>,
) -> main_gui::Description {
    match (data, desc) {
        (Some(data), Some(desc)) => {
            let kind = get_kind_str(&data);
            let mut buffer = vec![format!("{} {:?}", kind, data.get_id())];
            if let Some(action) = desc.get_action() {
                buffer.push(get_action_string(action));
            }
            if let Some(target_id) = desc.get_nav_move_to_target() {
                buffer.push(format!("target id: {:?}", target_id));
            }
            if let Some(cargo) = desc.get_cargo() {
                buffer.extend(get_cargo_str(wares, cargo));
            }
            if let Some(factory) = desc.get_factory() {
                if factory.is_producing() {
                    buffer.push("producing".to_string());
                }
            }
            if let Some(shipyard) = desc.get_shipyard() {
                if shipyard.is_producing() {
                    buffer.push("producing ship".to_string());
                }
            }
            if let Some(target_sector) = jump_target_sector {
                buffer.push(format!("jump to {}", target_sector));
            }
            main_gui::Description::Obj {
                title: desc.get_label().to_string(),
                desc: buffer.join("\n"),
            }
        }
        _ => main_gui::Description::None,
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

pub fn generate_sectorview_updates(state: &State, sector_id: Id) -> sector_view::RefreshParams {
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

    let mut params = sector_view::RefreshParams::default();
    params.updates = updates;
    params
}
