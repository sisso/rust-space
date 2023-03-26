use crate::graphics::AstroModel;
use crate::state::State;
use commons::unwrap_or_continue;
use godot::engine::node::InternalMode;
use godot::engine::{global, Engine};
use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::Location;
use specs::prelude::*;

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl SectorView {
    pub fn update_sector(&mut self, state: &State) {
        godot_print!("SectorView.draw_sector");

        let game = state.game.borrow();
        let entities = game.world.entities();
        // let sectors = game.world.read_storage::<Sector>();
        // let (sector_id, _) = (&entities, &sectors).join().next().unwrap();
        //
        // let sectors_index = game.world.read_resource::<EntityPerSectorIndex>();
        // let objects_at_sector = match sectors_index.index.get(&sector_id) {
        //     Some(list) => {
        //         godot_warn!("sector {} has no index, skipping draw sector", sector_id.id());
        //         list
        //     },
        //     None => return,
        // };
        //
        // let bs = BitSet::from_iter(objects_at_sector.iter().map(|e| e.id()));

        // add objects
        let locations = game.world.read_storage::<Location>();
        let fleets = game.world.read_storage::<Fleet>();
        let astros = game.world.read_storage::<AstroBody>();

        let fleet_color = crate::utils::color_red();
        let astro_color = crate::utils::color_blue();
        let star_color = crate::utils::color_yellow();

        for (e, l, f, a) in (&entities, &locations, fleets.maybe(), astros.maybe()).join() {
            godot_print!("id {}", e.id());

            let pos = unwrap_or_continue!(l.get_pos());

            if f.is_some() {
            } else if let Some(astro) = a {
                let color = match astro.kind {
                    AstroBodyKind::Star => star_color,
                    AstroBodyKind::Planet => astro_color,
                };

                let scale = 0.25;
                let mut model = AstroModel::new_alloc();
                let mut base: Gd<Node2D> = model.upcast();
                base.set_name(format!("Astro {}", e.id()).into());
                base.set_scale(Vector2::new(scale, scale));
                base.set_position(Vector2::new(pos.x, pos.y));
                godot_print!("create model at {} {}", pos.x, pos.y);
                self.base
                    .add_child(base.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
            }
        }

        godot_print!("done")
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
        let change = 10.0 * delta as f32;
        let scale_change = 0.01f32;

        let input = Input::singleton();
        if input.is_key_pressed(global::Key::KEY_W) {
            self.base.translate(Vector2::new(0.0, change));
        }
        if input.is_key_pressed(global::Key::KEY_S) {
            self.base.translate(Vector2::new(0.0, -change));
        }
        if input.is_key_pressed(global::Key::KEY_A) {
            self.base.translate(Vector2::new(change, 0.0));
        }
        if input.is_key_pressed(global::Key::KEY_D) {
            self.base.translate(Vector2::new(-change, 0.0));
        }

        if input.is_key_pressed(global::Key::KEY_Q) {
            self.base
                .apply_scale(Vector2::new(1.0 - scale_change, 1.0 - scale_change));
        }
        if input.is_key_pressed(global::Key::KEY_E) {
            self.base
                .apply_scale(Vector2::new(1.0 + scale_change, 1.0 + scale_change));
        }
    }
}
