use crate::main_gui::{LabeledEntity, MainGui};
use crate::sector_view::SectorView;
use crate::state::{State, StateScreen};
use godot::bind::{godot_api, GodotClass};
use godot::engine::{Engine, Node, NodeExt, NodeVirtual};
use godot::log::{godot_print, godot_warn};
use godot::obj::Base;
use godot::prelude::*;
use rand::random;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::{EntityPerSectorIndex, Location};
use space_domain::game::sectors::{Sector, SectorId, Sectors};
use space_domain::game::{scenery_random, Game};
use space_domain::utils::DeltaTime;
use specs::prelude::*;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    runtime: Option<Runtime>,
    #[base]
    base: Base<Node>,
}

struct Runtime {
    state: State,
    sector_view: Gd<SectorView>,
    gui: Gd<MainGui>,
}

#[godot_api]
impl GameApi {
    pub fn get_instance<T>(provided: Gd<T>) -> Gd<GameApi>
    where
        T: Inherits<Node>,
    {
        let node = provided.upcast();
        node.get_node_as::<GameApi>("/root/GameApi")
    }

    pub fn on_click_sector(&mut self, sector_id: SectorId) {
        let runtime = self.runtime.as_mut().unwrap();
        runtime.state.screen = StateScreen::Sector(sector_id);

        runtime.sector_view.bind_mut().update(&runtime.state);
    }

    pub fn update_gui(&mut self) {
        let (sectors, fleets) = {
            let game = self.runtime.as_ref().unwrap().state.game.borrow();

            let entities = game.world.entities();
            let sectors_storage = game.world.read_storage::<Sector>();

            let sectors: Vec<_> = (&entities, &sectors_storage)
                .join()
                .map(|(e, s)| LabeledEntity {
                    id: e,
                    label: format!("{} {}", s.coords.x, s.coords.y),
                })
                .collect();

            let fleets_storage = game.world.read_storage::<Fleet>();
            let fleets: Vec<String> = fleets_storage
                .as_slice()
                .iter()
                .enumerate()
                .map(|(i, fleet)| format!("Fleet {}", i))
                .collect();

            (sectors, fleets)
        };

        let runtime = self.runtime.as_mut().unwrap();
        let mut gui = runtime.gui.bind_mut();
        gui.show_sectors(sectors);
        gui.show_fleets(fleets);
    }

    pub fn draw_sector(&mut self) {
        let runtime = self.runtime.as_mut().unwrap();
        let state = &runtime.state;
        let mut sv = runtime.sector_view.bind_mut();
        sv.update(state);
    }

    pub fn recenter(&mut self) {
        self.runtime
            .as_mut()
            .unwrap()
            .sector_view
            .bind_mut()
            .recenter();
    }
}

#[godot_api]
impl NodeVirtual for GameApi {
    fn init(base: Base<Node>) -> Self {
        if Engine::singleton().is_editor_hint() {
            GameApi {
                runtime: None,
                base: base,
            }
        } else {
            GameApi {
                runtime: None,
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
            let sector_view = self
                .try_get_node_as("/root/GameApi/SectorView")
                .expect("SectorView not found");
            let gui = self
                .try_get_node_as("/root/GameApi/MainGui")
                .expect("MainGui not found");

            let state = State::new();
            self.runtime = Some(Runtime {
                state,
                sector_view,
                gui,
            });

            self.update_gui();
            self.draw_sector();
            self.recenter();
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        {
            let mut game = self.runtime.as_mut().unwrap().state.game.borrow_mut();
            game.tick(DeltaTime::from(delta as f32));
        }

        self.draw_sector();
    }
}
