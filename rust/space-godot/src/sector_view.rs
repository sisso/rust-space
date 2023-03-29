use crate::graphics::AstroModel;
use crate::state::State;
use commons::unwrap_or_continue;
use godot::engine::node::InternalMode;
use godot::engine::{global, Engine};
use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::{EntityPerSectorIndex, Location};
use space_domain::game::sectors::{Sector, SectorId};
use specs::prelude::*;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl SectorView {
    pub fn update_sector(&mut self, state: &State, sector_id: Option<SectorId>) {
        godot_print!("SectorView.draw_sector");

        let game = state.game.borrow();
        let entities = game.world.entities();
        let sectors = game.world.read_storage::<Sector>();

        let sector_id = match sector_id {
            Some(id) => id,
            None => {
                let (sector_id, _) = (&entities, &sectors).join().next().unwrap();
                sector_id
            }
        };

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

        let bs = BitSet::from_iter(objects_at_sector.iter().map(|e| e.id()));

        // clear first
        crate::utils::clear(self.base.share());

        // add objects
        let locations = game.world.read_storage::<Location>();
        let fleets = game.world.read_storage::<Fleet>();
        let astros = game.world.read_storage::<AstroBody>();

        let fleet_color = crate::utils::color_red();
        let astro_color = crate::utils::color_blue();
        let star_color = crate::utils::color_yellow();

        for (_, e, l, f, a) in (&bs, &entities, &locations, fleets.maybe(), astros.maybe()).join() {
            let pos = unwrap_or_continue!(l.get_pos());

            let scale = 0.25;
            let scale_v = Vector2::new(scale, scale);

            if f.is_some() {
                let mut model = AstroModel::new_alloc();
                model.bind_mut().set_color(fleet_color);

                let mut base: Gd<Node2D> = model.upcast();
                base.set_name(format!("Fleet {}", e.id()).into());
                base.set_scale(scale_v);
                base.set_position(Vector2::new(pos.x, pos.y));
            } else if let Some(astro) = a {
                let color = match astro.kind {
                    AstroBodyKind::Star => star_color,
                    AstroBodyKind::Planet => astro_color,
                };

                let mut model = AstroModel::new_alloc();
                model.bind_mut().set_color(color);

                let mut base: Gd<Node2D> = model.upcast();
                base.set_name(format!("Astro {}", e.id()).into());
                base.set_scale(scale_v);
                base.set_position(Vector2::new(pos.x, pos.y));
                godot_print!("create model at {} {}", pos.x, pos.y);
                self.base
                    .add_child(base.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
            }
        }
    }

    pub fn recenter(&mut self) {
        self.base.translate(Vector2::new(600.0, 350.0));
        self.base.set_scale(Vector2::new(50.0, 50.0))
    }
}

#[godot_api]
impl Node2DVirtual for SectorView {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        Self { base }
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
