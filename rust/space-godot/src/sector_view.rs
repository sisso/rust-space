use crate::godot_utils;
use crate::godot_utils::V2Vec;
use crate::graphics::{AstroModel, OrbitModel, SelectedModel};
use crate::sector_view::SectorViewState::Selected;
use commons::math::V2;
use godot::classes::{
    Control, Engine, IControl, InputEvent, InputEventMouseButton, InputEventMouseMotion,
};
use godot::global;
use godot::global::{Key, MouseButton};
use godot::prelude::*;
use space_domain::game::objects::ObjId;
use std::collections::{HashMap, HashSet};

const MODEL_SCALE: f32 = 0.1;

#[derive(Debug, PartialEq)]
pub enum SectorViewState {
    None,
    Selected(ObjId),
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
    bodies_model: HashMap<ObjId, Gd<Node2D>>,
    orbits_model: HashMap<ObjId, Gd<Node2D>>,

    #[export]
    selected_model: Option<Gd<SelectedModel>>,

    #[export]
    build_plot_model: Option<Gd<SelectedModel>>,

    frame_selected_id: Option<ObjId>,
    frame_build_plot: Option<Vector2>,

    #[export]
    fake_data: bool,

    #[export]
    objects: Option<Gd<Node2D>>,

    #[base]
    base: Base<Control>,
}

#[derive(Debug, Default)]
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
        id: ObjId,
        pos: V2,
        kind: ObjKind,
    },
    Orbit {
        id: ObjId,
        pos: V2,
        parent_pos: V2,
        radius: f32,
    },
}

#[godot_api]
impl SectorView {
    pub fn get_selected_id(&self) -> Option<ObjId> {
        match self.state {
            SectorViewState::Selected(id) => Some(id),
            _ => None,
        }
    }

    pub fn take_selected_id(&mut self) -> Option<ObjId> {
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
                        self.objects
                            .as_mut()
                            .unwrap()
                            .add_child(model.upcast::<Node2D>());
                    }
                    current_entities.insert(id);
                }

                Update::Orbit {
                    id,
                    parent_pos,
                    radius,
                    ..
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
                                godot_utils::color_white(),
                                parent_pos.as_vector2(),
                            );
                            self.objects
                                .as_mut()
                                .unwrap()
                                .add_child(model.clone().upcast::<Node2D>());
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

        if input.is_key_pressed(Key::W) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(0.0, speed));
        }
        if input.is_key_pressed(Key::S) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(0.0, -speed));
        }
        if input.is_key_pressed(Key::A) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(speed, 0.0));
        }
        if input.is_key_pressed(Key::D) {
            self.objects
                .as_mut()
                .unwrap()
                .translate(Vector2::new(-speed, 0.0));
        }

        if input.is_key_pressed(Key::Q) {
            self.objects
                .as_mut()
                .unwrap()
                .apply_scale(Vector2::new(1.0 - scale_speed, 1.0 - scale_speed));
        }
        if input.is_key_pressed(Key::E) {
            self.objects
                .as_mut()
                .unwrap()
                .apply_scale(Vector2::new(1.0 + scale_speed, 1.0 + scale_speed));
        }
    }

    fn remove_missing(&mut self, current_entities: HashSet<ObjId>) {
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
                        .remove_child(orbit_model.clone().upcast::<Node2D>());
                    orbit_model.queue_free();
                }

                // godot_print!("removing object {:?}", entity);
                self.objects
                    .as_mut()
                    .unwrap()
                    .remove_child(node.clone().upcast::<Node2D>());
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

    pub fn find_nearest(&self, local_pos: Vector2, min_distance: f32) -> Option<ObjId> {
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

        log::debug!("updated state to {:?}", self.state);
    }

    fn set_selected(&mut self, id: Option<ObjId>) {
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
                log::debug!("removing selected marker model");
                parent.remove_child(gd.upcast::<Node>())
            });

        if let Some(id) = id {
            // update selection

            // attach the it as child of new parent, if exists
            if let Some(target_model) = self.bodies_model.get(&id) {
                target_model.clone().add_child(
                    self.selected_model
                        .as_mut()
                        .unwrap()
                        .clone()
                        .upcast::<Node>(),
                );
                self.selected_model.as_mut().unwrap().show();
                self.state = SectorViewState::Selected(id);
                log::debug!("setting selected marker model to {:?}", id);
            } else {
                godot_warn!("selected object {:?} has no model, ignoring", id);
                self.selected_model.as_mut().unwrap().hide();
                log::debug!("selected obj has no model {:?}, ignoring", id);
            }
        } else {
            // if none object was provided, hide it
            self.selected_model.as_mut().unwrap().hide();
            self.state = SectorViewState::None;
            log::debug!("no selected id provided, hidding the marker");
        }
    }

    fn set_build_plot(&mut self, enabled: bool) {
        if enabled {
            log::debug!("enabling plot build");
            if !self.build_plot_model.as_mut().unwrap().is_visible() {
                log::debug!("showing plot build cursor");
                self.build_plot_model.as_mut().unwrap().show();
            }
        } else {
            log::debug!("disabling plot build");
            if self.build_plot_model.as_mut().unwrap().is_visible() {
                log::debug!("hide plot build cursor");
                self.build_plot_model.as_mut().unwrap().hide();
            }
        }
    }

    fn to_local(&self, global_pos: Vector2) -> Vector2 {
        let mouse_local_pos = self.objects.as_ref().unwrap().to_local(global_pos);
        mouse_local_pos
    }

    fn get_local_mouse_pos(&self) -> Vector2 {
        let mouse_pos = self.objects.as_ref().unwrap().get_global_mouse_position();
        self.to_local(mouse_pos)
    }

    fn handle_mouse_down(&mut self, mouse_down: Gd<InputEventMouseButton>) {
        let local_mouse_pos = self.to_local(mouse_down.get_global_position());
        let is_left_button = mouse_down.get_button_index() == MouseButton::LEFT;
        let is_right_button = mouse_down.get_button_index() == MouseButton::RIGHT;
        match &self.state {
            SectorViewState::Plotting { .. } if is_left_button => {
                log::debug!("set plotting position to {:?}", local_mouse_pos);
                self.frame_build_plot = Some(local_mouse_pos);
            }
            SectorViewState::Plotting { .. } if is_right_button => {
                log::debug!("canceling plotting");
                self.set_state(SectorViewState::None);
            }
            _ if is_left_button => {
                let nearest_id = self.find_nearest(local_mouse_pos, 1.0);
                if let Some(id) = nearest_id {
                    log::debug!("set selected {:?}", id);
                    self.frame_selected_id = Some(id);
                    let new_state = SectorViewState::Selected(id);
                    self.set_state(new_state);
                } else {
                    log::debug!("set selected none");
                    self.set_state(SectorViewState::None);
                }
            }
            _ => {}
        }
    }

    fn handle_mouse_move(&mut self, mouse_move: Gd<InputEventMouseMotion>) {
        match &self.state {
            SectorViewState::Plotting { .. } => {
                let local_mouse_pos = self.to_local(mouse_move.get_global_position());
                if let Some(gd) = self.build_plot_model.clone() {
                    let mut node = gd.upcast::<Node2D>();
                    node.set_position(local_mouse_pos);
                }
                // .as_mut()
                // .unwrap()
                // .bind_mut()
                // .set_position(local_mouse_pos);
            }
            _ => {}
        }
    }
}

#[godot_api]
impl IControl for SectorView {
    fn init(base: Base<Control>) -> Self {
        Self {
            state: SectorViewState::None,
            bodies_model: Default::default(),
            orbits_model: Default::default(),
            selected_model: None,
            build_plot_model: None,
            objects: None,
            frame_build_plot: None,
            frame_selected_id: None,
            fake_data: false,
            base,
        }
    }

    fn ready(&mut self) {
        godot_print!("ready with fake data {:?}", self.fake_data);

        let mut selected_model = new_select_model("selected", godot_utils::color_cyan(), false);
        selected_model.hide();

        let mut build_plot_model =
            new_select_model("plot cursor", godot_utils::color_bright_blue(), true);
        build_plot_model.hide();
        self.objects
            .as_mut()
            .unwrap()
            .add_child(build_plot_model.clone().upcast::<Node2D>());

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
        let maybe_mouse_down: Result<Gd<InputEventMouseButton>, _> = event.clone().try_cast();
        log::trace!("get_input");
        if let Ok(mouse_down) = maybe_mouse_down {
            log::trace!("get_input mouse down");
            self.handle_mouse_down(mouse_down);
        }

        let maybe_mouse_move: Result<Gd<InputEventMouseMotion>, _> = event.try_cast();
        if let Ok(mouse_move) = maybe_mouse_move {
            log::trace!("get_input mouse move");
            self.handle_mouse_move(mouse_move);
        }
    }
}

/// when default_scale should be false when object will be child of an already scaled object
fn new_select_model(name: &str, color: Color, default_scale: bool) -> Gd<SelectedModel> {
    // let mut model = SelectedModel::default();
    let mut model: Gd<SelectedModel> = SelectedModel::new_alloc();
    // let mut model: Gd<SelectedModel> = Gd::<SelectedModel>::default();
    model.bind_mut().set_color(color);

    let mut base = model.clone().upcast::<Node2D>();
    base.set_name(name.into());

    if default_scale {
        base.set_scale(Vector2::ONE * MODEL_SCALE);
    }

    model
}

fn resolve_model_for_kind(id: ObjId, pos: V2, kind: ObjKind) -> Gd<Node2D> {
    let fleet_color = godot_utils::color_red();
    let astro_color = godot_utils::color_green();
    let jump_color = godot_utils::color_blue();
    let station_color = godot_utils::color_light_gray();

    if kind.fleet {
        new_model(format!("Fleet {:?}", id), pos.as_vector2(), fleet_color)
    } else if kind.jump {
        new_model(format!("Jump {:?}", id), pos.as_vector2(), jump_color)
    } else if kind.station {
        new_model(format!("Station {:?}", id), pos.as_vector2(), station_color)
    } else if kind.astro && kind.astro_star {
        new_model(
            format!("Star {:?}", id),
            pos.as_vector2(),
            godot_utils::color_yellow(),
        )
    } else if kind.astro {
        new_model(format!("Astro {:?}", id), pos.as_vector2(), astro_color)
    } else if kind.asteroid {
        new_model(
            format!("Asteroid {:?}", id),
            pos.as_vector2(),
            godot_utils::color_brown(),
        )
    } else {
        new_model(format!("Unknown {:?}", id), pos.as_vector2(), astro_color)
    }
}

fn new_model(name: String, pos: Vector2, color: Color) -> Gd<Node2D> {
    let mut model: Gd<AstroModel> = AstroModel::new_alloc();
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
    model.bind_mut().set_color(color);
    model.set_name(name.into());

    let mut base: Gd<Node2D> = model.upcast();
    base.set_scale(scale_v);
    base.set_position(pos);

    base
}
