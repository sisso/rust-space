use crate::graphics::AstroModel;
use crate::main_gui::{LabeledEntity, MainGui};
use crate::sector_view::SectorView;
use crate::state::State;
use commons::math::Transform2;
use commons::unwrap_or_continue;
use godot::bind::{godot_api, GodotClass};
use godot::builtin::GodotString;
use godot::engine::node::InternalMode;
use godot::engine::{Engine, Node, NodeExt, NodeVirtual};
use godot::log::{godot_print, godot_warn};
use godot::obj::Base;
use godot::prelude::*;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::{EntityPerSectorIndex, Location};
use space_domain::game::sectors::{Sector, Sectors};
use space_domain::game::{scenery_random, Game};
use specs::prelude::*;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    state: Option<State>,
    sector_view: Option<Gd<SectorView>>,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl GameApi {}

impl GameApi {
    pub fn update_gui(&mut self) {
        let game = self.get_game();

        godot_print!("GameApi.update_gui");
        let mut gui = self
            .base
            .get_parent()
            .expect("no parent")
            .find_child("MainGui".into(), true, false)
            .expect("MainGui not found in parent")
            .cast::<MainGui>();
        let mut gui = gui.bind_mut();

        let entities = game.world.entities();
        let sectors_storage = game.world.read_storage::<Sector>();

        let sectors: Vec<_> = (&entities, &sectors_storage)
            .join()
            .map(|(e, s)| LabeledEntity {
                id: e,
                label: format!("{} {}", s.coords.x, s.coords.y),
            })
            .collect();
        gui.show_sectors(sectors);

        let fleets_storage = game.world.read_storage::<Fleet>();
        let fleets: Vec<String> = fleets_storage
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, fleet)| format!("Fleet {}", i))
            .collect();
        gui.show_fleets(fleets);

        godot_print!("GameApi.update_gui end");
    }

    pub fn draw_sector(&mut self) {
        godot_print!("GodotApi.draw_sector");
        let mut sv = self
            .sector_view
            .as_mut()
            .expect("sector_view not defined")
            .bind_mut();

        let state = self.state.as_ref().expect("state not defined");
        sv.update_sector(state);
        sv.recenter();
    }

    fn get_game(&self) -> Ref<'_, Game> {
        self.state
            .as_ref()
            .expect("state not initialized")
            .game
            .borrow()
    }
}

#[godot_api]
impl NodeVirtual for GameApi {
    fn init(base: Base<Node>) -> Self {
        if Engine::singleton().is_editor_hint() {
            GameApi {
                state: None,
                sector_view: None,
                base: base,
            }
        } else {
            let state = State::new();
            GameApi {
                state: Some(state),
                sector_view: None,
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        self.sector_view = self
            .get_parent()
            .expect("no parent found")
            .find_child("SectorView".into(), true, true)
            .map(|i| i.cast());

        // godot_print!("ready {} {}", self.value, self.value_str);
        if Engine::singleton().is_editor_hint() {
        } else {
            godot_print!("ready");
            self.update_gui();
            self.draw_sector();
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
