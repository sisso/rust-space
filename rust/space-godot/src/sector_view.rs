use crate::graphics::AstroModel;
use crate::state::{State, StateScreen};
use commons::unwrap_or_continue;

use godot::engine::node::InternalMode;
use godot::engine::{global, Engine};
use godot::prelude::*;

use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::{EntityPerSectorIndex, Location};
use space_domain::game::sectors::{Jump, SectorId};
use space_domain::game::station::Station;
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
struct ShowSectorState {
    bodies: HashMap<Entity, Gd<Node2D>>,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    show_sector_state: ShowSectorState,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl SectorView {
    pub fn update(&mut self, state: &State) {
        match state.screen {
            StateScreen::Sector(sector_id) => self.update_sector(state, sector_id),
            _ => todo!("not implemented"),
        }
    }
    fn update_sector(&mut self, state: &State, sector_id: SectorId) {
        let game = state.game.borrow();
        let entities = game.world.entities();

        let sectors_index = game.world.read_resource::<EntityPerSectorIndex>();
        let objects_at_sector = match sectors_index.index.get(&sector_id) {
            Some(list) if !list.is_empty() => list,
            Some(list) => {
                godot_warn!("sector {} has index, but sector is empty", sector_id.id());
                list
            }
            None => {
                godot_warn!(
                    "sector {} has no index, skipping draw sector",
                    sector_id.id()
                );
                return;
            }
        };
        let sector_entities_bitset = BitSet::from_iter(objects_at_sector.iter().map(|e| e.id()));

        // add objects
        let locations = game.world.read_storage::<Location>();
        let fleets = game.world.read_storage::<Fleet>();
        let astros = game.world.read_storage::<AstroBody>();
        let stations = game.world.read_storage::<Station>();
        let jumps = game.world.read_storage::<Jump>();

        let fleet_color = crate::utils::color_red();
        let astro_color = crate::utils::color_green();
        let star_color = crate::utils::color_yellow();
        let jump_color = crate::utils::color_blue();
        let station_color = crate::utils::color_light_gray();

        let mut current_entities = HashSet::new();

        // add and update entities
        for (_, e, l, f, a, s, j) in (
            &sector_entities_bitset,
            &entities,
            &locations,
            fleets.maybe(),
            astros.maybe(),
            stations.maybe(),
            jumps.maybe(),
        )
            .join()
        {
            let pos = unwrap_or_continue!(l.get_pos());
            let pos = Vector2::new(pos.x, pos.y);

            if let Some(node) = self.show_sector_state.bodies.get_mut(&e) {
                node.set_position(pos);
                current_entities.insert(e);
                continue;
            }

            let model = if f.is_some() {
                Some(Self::new_model(
                    format!("Fleet {}", e.id()),
                    pos,
                    fleet_color,
                ))
            } else if let Some(astro) = a {
                let color = match astro.kind {
                    AstroBodyKind::Star => star_color,
                    AstroBodyKind::Planet => astro_color,
                };
                Some(Self::new_model(format!("Astro {}", e.id()), pos, color))
            } else if s.is_some() {
                Some(Self::new_model(
                    format!("Jump {}", e.id()),
                    pos,
                    station_color,
                ))
            } else if j.is_some() {
                Some(Self::new_model(format!("Jump {}", e.id()), pos, jump_color))
            } else {
                None
            };

            let model = unwrap_or_continue!(model);

            self.show_sector_state.bodies.insert(e, model.share());
            self.base
                .add_child(model.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
            current_entities.insert(e);
        }

        // remove non existing entities
        self.show_sector_state.bodies.retain(|k, v| {
            if current_entities.contains(k) {
                true
            } else {
                self.base.remove_child(v.share().upcast());
                v.queue_free();
                false
            }
        });
    }

    fn new_model(name: String, pos: Vector2, color: Color) -> Gd<Node2D> {
        let scale = 0.1;
        let scale_v = Vector2::new(scale, scale);

        let mut model = AstroModel::new_alloc();
        model.bind_mut().set_color(color);

        let mut base: Gd<Node2D> = model.upcast();
        base.set_name(name.into());
        base.set_scale(scale_v);
        base.set_position(pos);

        base
    }

    pub fn recenter(&mut self) {
        self.base.set_position(Vector2::new(600.0, 350.0));
        self.base.set_scale(Vector2::new(50.0, 50.0))
    }
}

#[godot_api]
impl Node2DVirtual for SectorView {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        Self {
            show_sector_state: Default::default(),
            base,
        }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        // TODO: fix change relative to current screen scale
        let speed = 100.0 * delta as f32;
        let scale_speed = 0.02f32;

        let input = Input::singleton();
        if input.is_key_pressed(global::Key::KEY_W) {
            self.base.translate(Vector2::new(0.0, speed));
        }
        if input.is_key_pressed(global::Key::KEY_S) {
            self.base.translate(Vector2::new(0.0, -speed));
        }
        if input.is_key_pressed(global::Key::KEY_A) {
            self.base.translate(Vector2::new(speed, 0.0));
        }
        if input.is_key_pressed(global::Key::KEY_D) {
            self.base.translate(Vector2::new(-speed, 0.0));
        }

        if input.is_key_pressed(global::Key::KEY_Q) {
            self.base
                .apply_scale(Vector2::new(1.0 - scale_speed, 1.0 - scale_speed));
        }
        if input.is_key_pressed(global::Key::KEY_E) {
            self.base
                .apply_scale(Vector2::new(1.0 + scale_speed, 1.0 + scale_speed));
        }
    }
}
