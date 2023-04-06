use crate::graphics::{AstroModel, OrbitModel};
use crate::state::{State, StateScreen};
use commons::unwrap_or_continue;

use godot::engine::node::InternalMode;
use godot::engine::{global, Engine};
use godot::prelude::*;

use crate::utils::V2Vec;
use commons::math::V2;
use space_domain::game::astrobody::{AstroBody, AstroBodyKind, OrbitalPos};
use space_domain::game::fleets::Fleet;
use space_domain::game::locations::{EntityPerSectorIndex, Location, Locations};
use space_domain::game::sectors::{Jump, SectorId};
use space_domain::game::station::Station;
use space_domain::utils;
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
struct ShowSectorState {
    bodies: HashMap<Entity, Gd<Node2D>>,
    orbits: HashMap<Entity, Gd<Node2D>>,
}

#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct SectorView {
    state: ShowSectorState,

    #[base]
    base: Base<Node2D>,
}

pub enum ObjectKind {
    Fleet,
    Jump,
    Station,
    Astro,
    AstroStar,
}

pub enum Update {
    Obj {
        id: Entity,
        pos: V2,
        kind: ObjectKind,
    },
    Orbit {
        id: Entity,
        pos: V2,
        parent_pos: V2,
    },
}

#[godot_api]
impl SectorView {
    pub fn update(&mut self, state: &State) {
        match state.screen {
            StateScreen::Sector(sector_id) => {
                self.update_sector_objects(generate_sector_updates(state, sector_id))
            }
            _ => todo!("not implemented"),
        }
    }

    fn update_sector_objects(&mut self, updates: Vec<Update>) {
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
                } => {
                    let distance = parent_pos.distance(pos);

                    self.state
                        .orbits
                        .entry(id)
                        .and_modify(|model| {
                            model.set_scale(Vector2::ONE * distance);
                            model.set_position(parent_pos.as_vector2());
                        })
                        .or_insert_with(|| {
                            let model = new_orbit_model(
                                format!("orbit of {:?}", id.id()),
                                distance,
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

    fn remove_missing(&mut self, current_entities: HashSet<Entity>) {
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

fn generate_sector_updates(state: &State, sector_id: SectorId) -> Vec<Update> {
    let mut updates = vec![];

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
            return vec![];
        }
    };
    let sector_entities_bitset = BitSet::from_iter(objects_at_sector.iter().map(|e| e.id()));

    // add objects
    let locations = game.world.read_storage::<Location>();
    let fleets = game.world.read_storage::<Fleet>();
    let astros = game.world.read_storage::<AstroBody>();
    let stations = game.world.read_storage::<Station>();
    let jumps = game.world.read_storage::<Jump>();
    let orbits = game.world.read_storage::<OrbitalPos>();

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

        let kind = if f.is_some() {
            ObjectKind::Fleet
        } else if let Some(astro) = a {
            match astro.kind {
                AstroBodyKind::Star => ObjectKind::AstroStar,
                AstroBodyKind::Planet => ObjectKind::Astro,
            }
        } else if s.is_some() {
            ObjectKind::Station
        } else if j.is_some() {
            ObjectKind::Jump
        } else {
            // godot_warn!("unknown object {:?}", e);
            continue;
        };

        updates.push(Update::Obj { id: e, pos, kind })
    }

    for (_, e, o, l) in (&sector_entities_bitset, &entities, &orbits, &locations).join() {
        // get current and parent position and compute radius distance
        let self_pos = unwrap_or_continue!(l.get_pos());
        let parent = unwrap_or_continue!(Locations::resolve_space_position(&locations, o.parent));

        updates.push(Update::Orbit {
            id: e,
            pos: self_pos,
            parent_pos: parent.pos,
        });
    }

    updates
}

fn resolve_model_for_kind(id: Entity, pos: V2, kind: ObjectKind) -> Gd<Node2D> {
    let fleet_color = crate::utils::color_red();
    let astro_color = crate::utils::color_green();
    let star_color = crate::utils::color_yellow();
    let jump_color = crate::utils::color_blue();
    let station_color = crate::utils::color_light_gray();

    match kind {
        ObjectKind::Fleet => new_model(format!("Fleet {}", id.id()), pos.as_vector2(), fleet_color),
        ObjectKind::Jump => new_model(format!("Jump {}", id.id()), pos.as_vector2(), jump_color),
        ObjectKind::Station => new_model(
            format!("Station {}", id.id()),
            pos.as_vector2(),
            station_color,
        ),
        ObjectKind::AstroStar => {
            new_model(format!("Star {}", id.id()), pos.as_vector2(), star_color)
        }
        ObjectKind::Astro => new_model(format!("Astro {}", id.id()), pos.as_vector2(), astro_color),
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
