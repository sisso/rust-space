use crate::main_gui::{LabeledId, MainGui};
use crate::sector_view;
use crate::sector_view::SectorView;
use crate::state::{State, StateScreen};
use godot::obj::Gd;
use space_flap::Id;

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
        self.refresh_sector_view();
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
