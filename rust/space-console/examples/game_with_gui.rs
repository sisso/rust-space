extern crate space_domain;

use space_console::gui::{Gui, GuiObj, GuiObjKind, GuiSector, ShowSectorView};
use space_domain::ffi::FFIApi;
use space_domain::space_outputs_generated::space_data;

use space_domain::utils::V2;
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct SectorViewsImpl {
    sectors: HashMap<u32, GuiSector>,
    obj_index: HashMap<u32, u32>,
}

impl SectorViewsImpl {
    pub fn new() -> Self {
        SectorViewsImpl {
            sectors: Default::default(),
            obj_index: Default::default(),
        }
    }

    fn update(&mut self, outputs: space_data::Outputs) {
        if let Some(sectors) = outputs.sectors() {
            for sector in sectors {
                self.sectors.insert(
                    sector.id(),
                    GuiSector {
                        id: sector.id(),
                        label: format!("Sector {}", sector.id()),
                        objects: vec![],
                    },
                );
            }
        }

        for i in outputs.jumps().unwrap_or(&vec![]) {
            let v = self.sectors.get_mut(&i.sector_id()).unwrap();
            v.objects.push(GuiObj {
                id: i.id(),
                kind: GuiObjKind::JUMP,
                pos: SectorViewsImpl::from_v2(i.pos()),
            });
        }

        for i in outputs.entities_new().unwrap_or(&vec![]) {
            let sector_id = i.sector_id();

            let gui_sector = self.sectors.get_mut(&sector_id).unwrap();
            gui_sector.objects.push(GuiObj {
                id: i.id(),
                kind: SectorViewsImpl::from_kind(i.kind()),
                pos: SectorViewsImpl::from_v2(i.pos()),
            });

            self.obj_index.insert(i.id(), sector_id);
        }

        for e in outputs.entities_move().unwrap_or(&vec![]) {
            let sector_id = self.obj_index.get(&e.id()).unwrap();
            let gui_sector = self.sectors.get_mut(&sector_id).unwrap();

            for gui_obj in gui_sector.objects.iter_mut() {
                if gui_obj.id == e.id() {
                    gui_obj.pos = SectorViewsImpl::from_v2(e.pos());
                    break;
                }
            }
        }

        for i in outputs.entities_jump().unwrap_or(&vec![]) {
            let sector_id = self.obj_index.get(&i.id()).unwrap();
            let gui_sector = self.sectors.get_mut(&sector_id).unwrap();

            let index = gui_sector
                .objects
                .iter()
                .position(|j| j.id == i.id())
                .unwrap();
            let mut gui_obj = gui_sector.objects.remove(index);
            gui_obj.pos = SectorViewsImpl::from_v2(i.pos());

            let gui_sector = self.sectors.get_mut(&i.sector_id()).unwrap();
            gui_sector.objects.push(gui_obj);
            self.obj_index.insert(i.id(), i.sector_id());
        }
    }

    fn from_v2(v2: &space_data::V2) -> V2 {
        V2::new(v2.x(), v2.y())
    }

    fn from_kind(kind: space_data::EntityKind) -> GuiObjKind {
        match kind {
            space_data::EntityKind::Jump => GuiObjKind::JUMP,
            space_data::EntityKind::Fleet => GuiObjKind::SHIP,
            space_data::EntityKind::Asteroid => GuiObjKind::ASTEROID,
            space_data::EntityKind::Station => GuiObjKind::STATION,
        }
    }
}

impl ShowSectorView for SectorViewsImpl {
    fn get_sectors_id(&self) -> Vec<u32> {
        self.sectors.keys().map(|i| *i).collect()
    }

    fn get_sector(&self, sector_id: u32) -> &GuiSector {
        self.sectors.get(&sector_id).unwrap()
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut game_api = FFIApi::new();
    game_api.new_game();

    let time_rate = Duration::from_millis(100);

    let mut gui = Gui::new(time_rate)?;
    let mut sector_view = SectorViewsImpl::new();

    loop {
        let start = Instant::now();

        game_api.update(time_rate);

        game_api.get_inputs(|bytes| {
            let outputs = space_data::get_root_as_outputs(bytes.as_slice());
            sector_view.update(outputs);
        });

        gui.show_sectors(&sector_view);

        if gui.exit() {
            break;
        }

        // TODO: move it to own class
        let now = Instant::now();
        let delta = now - start;
        if delta < time_rate {
            let wait_time = time_rate - delta;
            eprintln!("gui - delta {:?}, wait_time: {:?}, ration: {:?}", delta, wait_time, time_rate.as_millis() as f64 / delta.as_millis() as f64);
            std::thread::sleep(wait_time);
        } else {
            eprintln!("gui - delta {:?}, wait_time: 0.0: missing time frame", delta);
        }
    }

    Ok(())
}
