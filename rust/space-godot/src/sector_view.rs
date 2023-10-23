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

const MODEL_SCALE: f32 = 0.1;

pub enum SectorViewState {
    None,
    Selected(Id),
    Plotting,
}

impl SectorViewState {
    pub fn is_none(&self) -> bool {
        match self {
            SectorViewState::None => true,
            _ => false,
        }
    }
    pub fn is_plot(&self) -> bool {
        match self {
            SectorViewState::Plotting => true,
            _ => false,
        }
    }
    pub fn is_selected(&self) -> bool {
        match self {
            SectorViewState::Selected(_) => true,
            _ => false,
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    state: SectorViewState,
    bodies_model: HashMap<Id, Gd<Node2D>>,
    orbits_model: HashMap<Id, Gd<Node2D>>,
    selected_model: Gd<SelectedModel>,
    build_plot_model: Gd<SelectedModel>,

    #[base]
    base: Base<Node2D>,
}

#[derive(Debug)]
pub struct ObjKind {
    pub fleet: bool,
    pub jump: bool,
    pub station: bool,
    pub asteroid: bool,
    pub astro: bool,
    pub astro_star: bool,
}

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct RefreshParams {
    pub updates: Vec<Update>,
}

#[godot_api]
impl SectorView {
    pub fn get_selected_id(&self) -> Option<Id> {
        match self.state {
            SectorViewState::Selected(id) => Some(id),
            _ => None,
        }
    }

    pub fn refresh(&mut self, params: RefreshParams) {
        let mut current_entities = HashSet::new();

        // add and update entities
        for update in params.updates {
            match update {
                Update::Obj { id, pos, kind } => {
                    if let Some(node) = self.bodies_model.get_mut(&id) {
                        // update existing object
                        node.set_position(pos.as_vector2());
                    } else {
                        // add model for new object
                        let model = resolve_model_for_kind(id, pos, kind);
                        self.bodies_model.insert(id, model.clone());
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
                    self.orbits_model
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
            .get_selected_id()
            .map(|id| !current_entities.contains(&id))
            .unwrap_or(false);

        if will_remove_selected {
            self.set_selected(None);
        }

        self.bodies_model.retain(|entity, node| {
            if current_entities.contains(entity) {
                true
            } else {
                if let Some(mut orbit_model) = self.orbits_model.remove(entity) {
                    // godot_print!("removing orbit {:?}", entity);
                    self.base.remove_child(orbit_model.clone().upcast());
                    orbit_model.queue_free();
                }

                // godot_print!("removing object {:?}", entity);
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
        for (id, gd) in &self.bodies_model {
            let ipos = gd.get_position();
            let idist = ipos.distance_squared_to(local_pos);
            if idist < dist {
                nid = Some(*id);
                dist = idist;
            }
        }

        nid
    }

    pub fn set_state(&mut self, new_state: SectorViewState) {
        match &new_state {
            SectorViewState::None => {
                self.set_build_plot(false);
                self.set_selected(None);
            }
            SectorViewState::Selected(id) => {
                self.set_build_plot(false);
                self.set_selected(Some(*id));
            }
            SectorViewState::Plotting => {
                self.set_build_plot(true);
                self.set_selected(None);
            }
        }

        self.state = new_state;
    }

    fn set_selected(&mut self, id: Option<Id>) {
        // do nothing if is still same object
        if id == self.get_selected_id() {
            return;
        }

        // detach the selected from previous parent
        self.selected_model.get_parent().map(|mut parent| {
            let gd = self.selected_model.clone();
            parent.remove_child(gd.upcast())
        });

        if let Some(id) = id {
            // update selection

            // attach the it as child of new parent, if exists
            if let Some(target_model) = self.bodies_model.get(&id) {
                target_model
                    .clone()
                    .add_child(self.selected_model.clone().upcast());
                self.selected_model.show();
                self.state = SectorViewState::Selected(id);
            } else {
                godot_warn!("selected object {:?} has no model, ignoring", id);
                self.selected_model.hide();
            }
        } else {
            // if none object was provided, hide it
            self.selected_model.hide();
            self.state = SectorViewState::None;
        }
    }

    fn set_build_plot(&mut self, enabled: bool) {
        if enabled {
            if !self.build_plot_model.is_visible() {
                self.build_plot_model.show();
            }
        } else {
            if self.build_plot_model.is_visible() {
                self.build_plot_model.hide();
            }
        }
    }
}

#[godot_api]
impl Node2DVirtual for SectorView {
    fn init(mut base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        let mut selected_model = new_select_model("selected", utils::color_cyan(), false);
        selected_model.hide();

        let mut build_plot_model =
            new_select_model("plot cursor", utils::color_bright_blue(), true);
        build_plot_model.hide();
        base.add_child(build_plot_model.clone().upcast());

        Self {
            state: SectorViewState::None,
            bodies_model: Default::default(),
            orbits_model: Default::default(),
            selected_model,
            build_plot_model,
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

        let mouse_local_pos = self.get_local_mouse_pos();

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

        match &self.state {
            SectorViewState::Plotting { .. } => {
                self.build_plot_model.set_position(mouse_local_pos);

                if input.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                    self.set_state(SectorViewState::None);
                }
            }
            _ => {
                if input.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    let new_state = self
                        .find_nearest(mouse_local_pos, 1.0)
                        .map(|id| SectorViewState::Selected(id))
                        .unwrap_or(SectorViewState::None);
                    self.set_state(new_state);
                }
            }
        }
    }
}

impl SectorView {
    fn get_local_mouse_pos(&self) -> Vector2 {
        let mouse_pos = self.base.get_global_mouse_position();
        let mouse_local_pos = self.base.to_local(mouse_pos);
        mouse_local_pos
    }
}

/// when default_scale should be false when object will be child of an already scaled object
fn new_select_model(name: &str, color: Color, default_scale: bool) -> Gd<SelectedModel> {
    let mut model = SelectedModel::new_alloc();
    model.bind_mut().set_color(color);

    let mut base: Gd<Node2D> = model.clone().upcast();
    base.set_name(name.into());

    if default_scale {
        base.set_scale(Vector2::ONE * MODEL_SCALE);
    }

    model
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
    let mut model = AstroModel::new_alloc();
    model.bind_mut().set_color(color);

    let mut base: Gd<Node2D> = model.upcast();
    base.set_name(name.into());
    base.set_scale(Vector2::ONE * MODEL_SCALE);
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
