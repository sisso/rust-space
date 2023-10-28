use std::collections::{HashMap, HashSet};

use godot::engine::global::MouseButton;
use godot::engine::{
    global, Control, ControlVirtual, Engine, InputEvent, InputEventMouseButton,
    InputEventMouseMotion,
};
use godot::prelude::*;

use commons::math::V2;
use space_flap::Id;

use crate::graphics::{AstroModel, OrbitModel, SelectedModel};
use crate::utils;
use crate::utils::V2Vec;

const MODEL_SCALE: f32 = 0.1;

#[derive(Debug, PartialEq)]
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
#[class(base = Control)]
pub struct SectorView {
    state: SectorViewState,
    bodies_model: HashMap<Id, Gd<Node2D>>,
    orbits_model: HashMap<Id, Gd<Node2D>>,
    selected_model: Option<Gd<SelectedModel>>,
    build_plot_model: Option<Gd<SelectedModel>>,
    objects: Option<Gd<Node2D>>,
    frame_selected_id: Option<Id>,
    frame_build_plot: Option<Vector2>,

    #[base]
    base: Base<Control>,
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

#[godot_api]
impl SectorView {
    pub fn get_selected_id(&self) -> Option<Id> {
        match self.state {
            SectorViewState::Selected(id) => Some(id),
            _ => None,
        }
    }

    pub fn take_selected_id(&mut self) -> Option<Id> {
        self.frame_selected_id.take()
    }

    pub fn take_build_plot(&mut self) -> Option<Vector2> {
        self.frame_build_plot.take()
    }

    pub fn refresh(&mut self, updates: Vec<Update>) {
        let mut current_entities = HashSet::new();

        // add and update entities
        for update in updates {
            match update {
                Update::Obj { id, pos, kind } => {
                    if let Some(node) = self.bodies_model.get_mut(&id) {
                        // update existing object
                        node.set_position(pos.as_vector2());
                    } else {
                        // add model for new object
                        let model = resolve_model_for_kind(id, pos, kind);
                        self.bodies_model.insert(id, model.clone());
                        self.objects.as_mut().unwrap().add_child(model.upcast());
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
                                utils::color_white(),
                                parent_pos.as_vector2(),
                            );
                            self.objects
                                .as_mut()
                                .unwrap()
                                .add_child(model.clone().upcast());
                            current_entities.insert(id);
                            model
                        });
                }
            }
        }

        // remove non existing entities
        self.remove_missing(current_entities);
    }

    fn process_input(&mut self, delta_seconds: f64) {
        // TODO: fix change relative to current screen scale
        let speed = 100.0 * delta_seconds as f32;
        let scale_speed = 0.02f32;

        let input = Input::singleton();
        if input.is_key_pressed(global::Key::KEY_W) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(0.0, speed));
        }
        if input.is_key_pressed(global::Key::KEY_S) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(0.0, -speed));
        }
        if input.is_key_pressed(global::Key::KEY_A) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(speed, 0.0));
        }
        if input.is_key_pressed(global::Key::KEY_D) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(-speed, 0.0));
        }

        if input.is_key_pressed(global::Key::KEY_Q) {
            self.objects
                .as_mut()
                .unwrap()
                .apply_scale(Vector2::new(1.0 - scale_speed, 1.0 - scale_speed));
        }
        if input.is_key_pressed(global::Key::KEY_E) {
            self.objects
                .as_mut()
                .unwrap()
                .apply_scale(Vector2::new(1.0 + scale_speed, 1.0 + scale_speed));
        }
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
                    self.objects
                        .as_mut()
                        .unwrap()
                        .remove_child(orbit_model.clone().upcast());
                    orbit_model.queue_free();
                }

                // godot_print!("removing object {:?}", entity);
                self.objects
                    .as_mut()
                    .unwrap()
                    .remove_child(node.clone().upcast());
                node.queue_free();
                false
            }
        });
    }

    pub fn recenter(&mut self) {
        self.objects
            .as_mut()
            .unwrap()
            .set_position(Vector2::new(500.0, 350.0));
        self.objects
            .as_mut()
            .unwrap()
            .set_scale(Vector2::new(50.0, 50.0))
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
        if self.state == new_state {
            return;
        }

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
        self.selected_model
            .as_mut()
            .unwrap()
            .get_parent()
            .map(|mut parent| {
                let gd = self.selected_model.as_mut().unwrap().clone();
                parent.remove_child(gd.upcast())
            });

        if let Some(id) = id {
            // update selection

            // attach the it as child of new parent, if exists
            if let Some(target_model) = self.bodies_model.get(&id) {
                target_model
                    .clone()
                    .add_child(self.selected_model.as_mut().unwrap().clone().upcast());
                self.selected_model.as_mut().unwrap().show();
                self.state = SectorViewState::Selected(id);
            } else {
                godot_warn!("selected object {:?} has no model, ignoring", id);
                self.selected_model.as_mut().unwrap().hide();
            }
        } else {
            // if none object was provided, hide it
            self.selected_model.as_mut().unwrap().hide();
            self.state = SectorViewState::None;
        }
    }

    fn set_build_plot(&mut self, enabled: bool) {
        if enabled {
            if !self.build_plot_model.as_mut().unwrap().is_visible() {
                self.build_plot_model.as_mut().unwrap().show();
            }
        } else {
            if self.build_plot_model.as_mut().unwrap().is_visible() {
                self.build_plot_model.as_mut().unwrap().hide();
            }
        }
    }
}

#[godot_api]
impl ControlVirtual for SectorView {
    fn init(mut base: Base<Control>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        Self {
            state: SectorViewState::None,
            bodies_model: Default::default(),
            orbits_model: Default::default(),
            selected_model: None,
            build_plot_model: None,
            objects: None,
            frame_build_plot: None,
            frame_selected_id: None,
            base,
        }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        let mut objects = self.base.get_node_as::<Node2D>("objects");

        let mut selected_model = new_select_model("selected", utils::color_cyan(), false);
        selected_model.hide();

        let mut build_plot_model =
            new_select_model("plot cursor", utils::color_bright_blue(), true);
        build_plot_model.hide();
        objects.add_child(build_plot_model.clone().upcast());

        self.objects = Some(objects);
        self.build_plot_model = Some(build_plot_model);
        self.selected_model = Some(selected_model);
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        self.process_input(delta);
    }

    fn gui_input(&mut self, event: Gd<InputEvent>) {
        let maybe_mouse_down: Option<Gd<InputEventMouseButton>> = event.clone().try_cast();
        if let Some(mouse_down) = maybe_mouse_down {
            // godot_print!(
            //     "sectorview receive input {:?} at {:?} with {:?}",
            //     mouse_down,
            //     self.to_local(mouse_down.get_position()),
            //     mouse_down.get_button_index()
            // );

            match &self.state {
                SectorViewState::Plotting { .. }
                    if mouse_down.get_button_index() == MouseButton::MOUSE_BUTTON_LEFT =>
                {
                    let global_mouse_pos = self.to_local(mouse_down.get_global_position());
                    godot_print!("set building plot pos {:?}", global_mouse_pos);
                    self.frame_build_plot = Some(global_mouse_pos);
                }
                SectorViewState::Plotting { .. }
                    if mouse_down.get_button_index() == MouseButton::MOUSE_BUTTON_RIGHT =>
                {
                    godot_print!("canceling building plot");
                    self.set_state(SectorViewState::None);
                }
                _ if mouse_down.get_button_index() == MouseButton::MOUSE_BUTTON_LEFT => {
                    let nearest_id =
                        self.find_nearest(self.to_local(mouse_down.get_global_position()), 1.0);

                    if let Some(id) = nearest_id {
                        self.frame_selected_id = Some(id);

                        let new_state = SectorViewState::Selected(id);
                        self.set_state(new_state);
                    } else {
                        self.set_state(SectorViewState::None);
                    }
                }
                _ => {}
            }

            return;
        }

        let maybe_mouse_move: Option<Gd<InputEventMouseMotion>> = event.try_cast();
        if let Some(mouse_move) = maybe_mouse_move {
            match &self.state {
                SectorViewState::Plotting { .. } => {
                    let local_mouse_pos = self.to_local(mouse_move.get_global_position());
                    self.build_plot_model
                        .as_mut()
                        .unwrap()
                        .set_position(local_mouse_pos);
                }
                _ => {}
            }
            return;
        }
    }
}

impl SectorView {
    fn to_local(&self, global_pos: Vector2) -> Vector2 {
        let mouse_local_pos = self.objects.as_ref().unwrap().to_local(global_pos);
        mouse_local_pos
    }

    fn get_local_mouse_pos(&self) -> Vector2 {
        let mouse_pos = self.objects.as_ref().unwrap().get_global_mouse_position();
        self.to_local(mouse_pos)
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
