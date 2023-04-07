use crate::graphics::{AstroModel, OrbitModel};
use crate::state::{State, StateScreen};

use godot::engine::node::InternalMode;
use godot::engine::{global, Engine};
use godot::prelude::*;

use crate::utils::V2Vec;
use commons::math::V2;
use commons::unwrap_or_continue;
use godot::private::callbacks::create;
use space_flap::{Id, ObjData};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
struct ShowSectorState {
    bodies: HashMap<Id, Gd<Node2D>>,
    orbits: HashMap<Id, Gd<Node2D>>,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    state: ShowSectorState,

    #[base]
    base: Base<Node2D>,
}

pub struct ObjKind {
    pub fleet: bool,
    pub jump: bool,
    pub station: bool,
    pub asteroid: bool,
    pub astro: bool,
}

pub enum Update {
    Obj {
        id: Id,
        pos: V2,
        kind: ObjKind,
    },
    Orbit {
        id: Id,
        pos: V2,
        parent_pos: V2,
        radius: f32,
    },
}

#[godot_api]
impl SectorView {
    pub fn refresh(&mut self, updates: Vec<Update>) {
        let mut current_entities = HashSet::new();

        // add and update entities
        for update in updates {
            match update {
                Update::Obj { id, pos, kind } => {
                    if let Some(node) = self.state.bodies.get_mut(&id) {
                        node.set_position(pos.as_vector2());
                    } else {
                        let model = resolve_model_for_kind(id, pos, kind);
                        self.state.bodies.insert(id, model.share());
                        self.base.add_child(
                            model.upcast(),
                            false,
                            InternalMode::INTERNAL_MODE_DISABLED,
                        );
                    }
                    current_entities.insert(id);
                }

                Update::Orbit {
                    id,
                    pos,
                    parent_pos,
                    radius,
                } => {
                    self.state
                        .orbits
                        .entry(id)
                        .and_modify(|model| {
                            model.set_scale(Vector2::ONE * radius);
                            model.set_position(parent_pos.as_vector2());
                        })
                        .or_insert_with(|| {
                            let model = new_orbit_model(
                                format!("orbit of {:?}", id),
                                radius,
                                crate::utils::color_white(),
                                parent_pos.as_vector2(),
                            );
                            self.base.add_child(
                                model.share().upcast(),
                                false,
                                InternalMode::INTERNAL_MODE_DISABLED,
                            );
                            current_entities.insert(id);
                            model
                        });
                }
            }
        }

        // remove non existing entities
        self.remove_missing(current_entities);
    }

    fn remove_missing(&mut self, current_entities: HashSet<Id>) {
        self.state.bodies.retain(|entity, node| {
            if current_entities.contains(entity) {
                true
            } else {
                if let Some(mut orbit_model) = self.state.orbits.remove(entity) {
                    godot_print!("removing orbit {:?}", entity);
                    self.base.remove_child(orbit_model.share().upcast());
                    orbit_model.queue_free();
                }

                godot_print!("removing object {:?}", entity);
                self.base.remove_child(node.share().upcast());
                node.queue_free();
                false
            }
        });
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
            state: Default::default(),
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

fn resolve_model_for_kind(id: Id, pos: V2, kind: ObjKind) -> Gd<Node2D> {
    let fleet_color = crate::utils::color_red();
    let astro_color = crate::utils::color_green();
    let jump_color = crate::utils::color_blue();
    let station_color = crate::utils::color_light_gray();

    if kind.fleet {
        new_model(format!("Fleet {}", id), pos.as_vector2(), fleet_color)
    } else if kind.jump {
        new_model(format!("Jump {}", id), pos.as_vector2(), jump_color)
    } else if kind.station {
        new_model(format!("Station {}", id), pos.as_vector2(), station_color)
    } else if kind.astro {
        new_model(format!("Astro {}", id), pos.as_vector2(), astro_color)
    } else if kind.asteroid {
        new_model(
            format!("Asteroid {}", id),
            pos.as_vector2(),
            crate::utils::color_brown(),
        )
    } else {
        new_model(format!("Unknown {}", id), pos.as_vector2(), astro_color)
    }
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

fn new_orbit_model(name: String, radius: f32, color: Color, pos: Vector2) -> Gd<Node2D> {
    let scale = radius;
    let scale_v = Vector2::new(scale, scale);

    let mut model = OrbitModel::new_alloc();
    {
        let mut model = model.bind_mut();
        model.set_color(color);
        model.set_name(name.into());
    }

    let mut base: Gd<Node2D> = model.upcast();
    base.set_scale(scale_v);
    base.set_position(pos);

    base
}
