use std::collections::{HashMap, HashSet};

use godot::engine::global::MouseButton;
use godot::engine::{global, Engine};
use godot::prelude::*;

use commons::math::V2;
use commons::unwrap_or_return;
use space_flap::Id;

use crate::graphics::{AstroModel, OrbitModel, SelectedModel};
use crate::utils;
use crate::utils::V2Vec;

#[derive(Debug)]
struct SelectedObject {
    model: Gd<Node2D>,
    id: Option<Id>,
}

#[derive(Debug)]
struct ShowSectorState {
    bodies_model: HashMap<Id, Gd<Node2D>>,
    orbits_model: HashMap<Id, Gd<Node2D>>,
    selected: SelectedObject,
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
    pub astro_star: bool,
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
    pub fn get_selected_id(&self) -> Option<Id> {
        self.state.selected.id
    }

    pub fn refresh(&mut self, updates: Vec<Update>) {
        let mut current_entities = HashSet::new();

        // add and update entities
        for update in updates {
            match update {
                Update::Obj { id, pos, kind } => {
                    if let Some(node) = self.state.bodies_model.get_mut(&id) {
                        node.set_position(pos.as_vector2());
                    } else {
                        let model = resolve_model_for_kind(id, pos, kind);
                        self.state.bodies_model.insert(id, model.clone());
                        self.base.add_child(model.upcast());
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
                        .orbits_model
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
                            self.base.add_child(model.clone().upcast());
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
        // remove selected before the object get removed
        let will_remove_selected = self
            .state
            .selected
            .id
            .as_ref()
            .map(|id| !current_entities.contains(id))
            .unwrap_or(false);

        if will_remove_selected {
            self.set_selected(None);
        }

        self.state.bodies_model.retain(|entity, node| {
            if current_entities.contains(entity) {
                true
            } else {
                if let Some(mut orbit_model) = self.state.orbits_model.remove(entity) {
                    godot_print!("removing orbit {:?}", entity);
                    self.base.remove_child(orbit_model.clone().upcast());
                    orbit_model.queue_free();
                }

                godot_print!("removing object {:?}", entity);
                self.base.remove_child(node.clone().upcast());
                node.queue_free();
                false
            }
        });
    }

    pub fn recenter(&mut self) {
        self.base.set_position(Vector2::new(600.0, 350.0));
        self.base.set_scale(Vector2::new(50.0, 50.0))
    }

    pub fn find_nearest(&self, local_pos: Vector2, min_distance: f32) -> Option<Id> {
        let mut nid = None;
        let mut dist = min_distance;
        for (id, gd) in &self.state.bodies_model {
            let ipos = gd.get_position();
            let idist = ipos.distance_squared_to(local_pos);
            if idist < dist {
                nid = Some(*id);
                dist = idist;
            }
        }

        nid
    }

    pub fn set_selected(&mut self, id: Option<Id>) {
        // do nothing if is still same object
        if id == self.state.selected.id {
            return;
        }

        // remove parent if exists
        self.state.selected.model.get_parent().map(|mut parent| {
            let gd = self.state.selected.model.clone();
            parent.remove_child(gd.upcast())
        });

        // update selection
        self.state.selected.id = id;
        if let Some(id) = self.state.selected.id {
            let model = unwrap_or_return!(self.state.bodies_model.get(&id));
            let mut model = model.clone();

            self.state.selected.model.show();

            model.add_child(self.state.selected.model.clone().upcast());
        }
    }
}

#[godot_api]
impl Node2DVirtual for SectorView {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        let mut selected_model = SelectedModel::new_alloc();
        selected_model
            .bind_mut()
            .set_color(utils::color_bright_cyan());

        let mut selected_model_base: Gd<Node2D> = selected_model.upcast();
        selected_model_base.set_name("selected".into());
        selected_model_base.hide();

        let state = ShowSectorState {
            bodies_model: Default::default(),
            orbits_model: Default::default(),
            selected: SelectedObject {
                model: selected_model_base,
                id: None,
            },
        };

        Self {
            state: state,
            base: base,
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

        if input.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let pos = self.base.get_global_mouse_position();
            let local_pos = self.base.to_local(pos);
            let nearest = self.find_nearest(local_pos, 1.0);
            self.set_selected(nearest);
        }
    }
}

fn resolve_model_for_kind(id: Id, pos: V2, kind: ObjKind) -> Gd<Node2D> {
    let fleet_color = utils::color_red();
    let astro_color = utils::color_green();
    let jump_color = utils::color_blue();
    let station_color = utils::color_light_gray();

    if kind.fleet {
        new_model(format!("Fleet {}", id), pos.as_vector2(), fleet_color)
    } else if kind.jump {
        new_model(format!("Jump {}", id), pos.as_vector2(), jump_color)
    } else if kind.station {
        new_model(format!("Station {}", id), pos.as_vector2(), station_color)
    } else if kind.astro && kind.astro_star {
        new_model(
            format!("Star {}", id),
            pos.as_vector2(),
            crate::utils::color_yellow(),
        )
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
        model.base.set_name(name.into());
    }

    let mut base: Gd<Node2D> = model.upcast();
    base.set_scale(scale_v);
    base.set_position(pos);

    base
}
