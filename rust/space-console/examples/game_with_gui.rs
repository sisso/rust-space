extern crate space_domain;

use space_console::gui::{Gui, GuiSector, ShowSectorView, GuiObj, GuiObjKind};
use space_domain::game_api::GameApi;
use std::time::Duration;

struct SectorViewsImpl<'a> {
    api: &'a GameApi,
}

impl<'a> SectorViewsImpl<'a> {
    pub fn new(api: &'a  GameApi) -> Self {
        SectorViewsImpl { api }
    }
}

impl<'a> ShowSectorView for SectorViewsImpl<'a> {
    fn get_sectors_len(&self) -> usize {
        self.api.get_game().sectors.list().len()
    }

    fn get_sector(&self, sector_index: usize) -> GuiSector {
        let game = self.api.get_game();
        let sector_id = game.sectors.list().get(sector_index).unwrap().clone();

        let objects = game.locations.find_at_sector(sector_id);

        let mut gui_objects: Vec<GuiObj> = objects.into_iter().map(|obj_id| {
            let pos = game.locations.get_location(&obj_id).unwrap().get_space().pos;

            let kind =
                if game.extractables.get_extractable(&obj_id).is_some() {
                    GuiObjKind::ASTEROID
                } else if game.objects.get(&obj_id).has_dock {
                    GuiObjKind::STATION
                } else {
                    GuiObjKind::SHIP
                };

            GuiObj {
                kind,
                pos
            }
        }).collect();

        for jump in game.sectors.get_jumps(sector_id) {
            gui_objects.push(
                GuiObj {
                    kind: GuiObjKind::JUMP,
                    pos: jump.pos,
                }
            );
        }

        GuiSector {
            label: format!("Sector {}", sector_id.0),
            objects: gui_objects
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut game_api = GameApi::new();
    game_api.new_game();

    let time_rate = Duration::from_millis(100);

    let mut gui = Gui::new(time_rate)?;

    loop {
        game_api.update(time_rate);

        let sector_view = SectorViewsImpl::new(&game_api);
        gui.show_sectors(&sector_view);

        if gui.exit() {
            break;
        }
    }

    Ok(())
}
