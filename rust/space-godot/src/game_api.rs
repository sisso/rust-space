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
use space_domain::game::sectors::{Sector, SectorId, Sectors};
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
    gui: Option<Gd<MainGui>>,
    #[base]
    base: Base<Node>,
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
        self.sector_view
            .as_mut()
            .unwrap()
            .bind_mut()
            .update_sector(self.state.as_ref().unwrap(), Some(sector_id));
    }

    pub fn update_gui(&mut self) {
        let (sectors, fleets) = {
            let game = self.get_game();

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

        let mut gui = self.gui.as_mut().expect("MainGui not provided").bind_mut();
        gui.show_sectors(sectors);
        gui.show_fleets(fleets);
    }

    pub fn draw_sector(&mut self) {
        let mut sv = self
            .sector_view
            .as_mut()
            .expect("sector_view not defined")
            .bind_mut();

        let state = self.state.as_ref().expect("state not defined");
        sv.update_sector(state, None);
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
                gui: None,
                base: base,
            }
        } else {
            let state = State::new();
            GameApi {
                state: Some(state),
                sector_view: None,
                gui: None,
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        self.sector_view = self.try_get_node_as("/root/GameApi/SectorView");
        self.gui = self.try_get_node_as("/root/GameApi/MainGui");

        if Engine::singleton().is_editor_hint() {
        } else {
            godot_print!("ready");
            self.gui.as_mut().unwrap().bind_mut().connect(
                "signal_on_click_sector".into(),
                Callable::from_object_method(self.base.share(), "on_click_sector"),
                0,
            );

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
