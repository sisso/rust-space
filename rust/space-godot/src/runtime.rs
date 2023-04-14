use godot::obj::Gd;

use space_flap::{Id, ObjAction, ObjActionKind, ObjCargo, ObjData, ObjDesc};

use crate::main_gui::{LabeledId, MainGui};
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
        self.state.game.update(delta_seconds as f32);
        self.refresh_sector_view();
    }

    pub fn change_sector(&mut self, sector_id: Id) {
        self.state.screen = StateScreen::Sector(sector_id);
    }

    pub fn refresh_sector_view(&mut self) {
        match &self.state.screen {
            StateScreen::Sector(sector_id) => {
                self.sector_view
                    .bind_mut()
                    .refresh(generate_sectorview_updates(&self.state, *sector_id));
            }
            _ => {
                todo!("not implemented")
            }
        }
    }

    pub fn update_gui(&mut self) {
        let mut sectors = vec![];
        for sector in self.state.game.get_sectors() {
            let (x, y) = sector.get_coords();
            let x = x as i32;
            let y = y as i32;
            sectors.push(LabeledId {
                id: sector.get_id(),
                label: format!("{} {}", x, y),
            })
        }

        let mut fleets = vec![];
        for fleet in self.state.game.get_fleets() {
            fleets.push(LabeledId {
                id: fleet.get_id(),
                label: format!("Fleet {}", fleet.get_id()),
            })
        }

        let mut gui = self.gui.bind_mut();
        gui.show_sectors(sectors);
        gui.show_fleets(fleets);
    }

    pub fn recenter(&mut self) {
        self.sector_view.bind_mut().recenter();
    }

    pub fn on_selected_entity(&mut self, id: Option<Id>) {
        if let Some(id) = id {
            let data = self.state.game.get_obj(id);
            let desc = self.state.game.get_obj_desc(id);
            let uidesc = describe_obj(data, desc);
            self.gui.bind_mut().show_selected_object(uidesc);
        } else {
            self.gui
                .bind_mut()
                .show_selected_object(main_gui::Description::None);
        }
    }
}

fn describe_obj(data: Option<ObjData>, desc: Option<ObjDesc>) -> main_gui::Description {
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
                buffer.extend(get_cargo_str(cargo));
            }
            if let Some(factory) = desc.get_factory() {
                if factory.is_producing() {
                    buffer.push("producing products".to_string());
                }
            }
            if let Some(shipyard) = desc.get_shipyard() {
                if shipyard.is_producing() {
                    buffer.push("producing ship".to_string());
                }
            }
            main_gui::Description::Obj {
                title: desc.get_label().to_string(),
                desc: buffer.join("\n"),
            }
        }
        _ => main_gui::Description::None,
    }
}

fn get_cargo_str(cargo: ObjCargo) -> Vec<String> {
    let mut b = vec![];
    b.push("Cargo:".to_string());
    if b.is_empty() {
        b.push("<empty>".to_string());
    } else {
        for (id, amount) in cargo.get_wares() {
            b.push(format!("- {}: {}", id, amount))
        }
    }
    b
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
