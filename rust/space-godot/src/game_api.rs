use crate::main_gui::MainGui;
use crate::state::State;
use commons::math::Transform2;
use godot::bind::{godot_api, GodotClass};
use godot::builtin::GodotString;
use godot::engine::{Engine, Node, NodeExt, NodeVirtual};
use godot::log::godot_print;
use godot::obj::Base;
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::EntityPerSectorIndex;
use space_domain::game::sectors::{Sector, Sectors};
use space_domain::game::{scenery_random, Game};
use specs::hibitset::BitSetLike;
use specs::{Entity, Join, WorldExt};
use std::cell::{Ref, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameApi {
    state: Option<State>,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl GameApi {}

impl GameApi {
    pub fn update_gui(&mut self) {
        let state = self.get_state();

        godot_print!("GameApi.update_gui");
        let gui = self
            .base
            .get_parent()
            .unwrap()
            .find_child("MainGui".into(), true, false);
        let mut gui = gui.unwrap().cast::<MainGui>();
        let gui = gui.bind_mut();

        let sectors_storage = state.world.read_storage::<Sector>();
        let sectors: Vec<String> = sectors_storage
            .as_slice()
            .iter()
            .map(|i| format!("{} {}", i.coords.x, i.coords.y))
            .collect();
        gui.show_sectors(sectors);

        let fleets_storage = state.world.read_storage::<Fleet>();
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
        let state = self.get_state();
        let entities = state.world.entities();
        let sectors = state.world.read_storage::<Sector>();
        let (sector_id, _) = (&entities, &sectors).join().next().unwrap();

        let sectors_index = state.world.read_resource::<EntityPerSectorIndex>();
        for entity in sectors_index.index.get(&sector_id).unwrap() {}

        // let sector_id = sectors.fetched_entities().create_iter().next().unwrap();
        // let mut sector_id = None;
        //
        // for (e, _) in (entities, sectors).join() {
        //     sector_id = Some(e);
        //     break;
        // }
        // let sector_id = sector_id.unwrap();
    }

    fn get_state(&self) -> Ref<'_, Game> {
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
                // value: 1,
                // value_str: Default::default(),
                base: base,
            }
        } else {
            let state = State::new();
            GameApi {
                state: Some(state),
                // value: 2,
                // value_str: Default::default(),
                base: base,
            }
        }
    }

    fn ready(&mut self) {
        // godot_print!("ready {} {}", self.value, self.value_str);
        godot_print!("ready 2");
        if Engine::singleton().is_editor_hint() {
        } else {
            godot_print!("ready self.update_gui");
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
