use godot::engine::node::InternalMode;
use godot::engine::{BoxContainer, Button, Engine, HBoxContainer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct MainGui {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl MainGui {
    pub fn show_sectors(&self) {
        // let container = self
        //     .base
        //     .get_node_as::<BoxContainer>("TabContainer/Main/SectorsVBoxContainer");
        let mut container = self
            .base
            .find_child("SectorsVBoxContainer".into(), true, true)
            .unwrap()
            .cast::<BoxContainer>();

        for c in container.get_children(true).iter_shared() {
            let mut n = c.cast::<Node>();
            container.remove_child(n.share());
            n.queue_free();
        }

        let mut h1 = HBoxContainer::new_alloc();
        let mut b1 = Button::new_alloc();
        b1.set_text("0 0".into());
        h1.add_child(b1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        let mut b1 = Button::new_alloc();
        b1.set_text("1 0".into());
        h1.add_child(b1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        container.add_child(h1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);

        let mut h1 = HBoxContainer::new_alloc();
        let mut b1 = Button::new_alloc();
        b1.set_text("0 1".into());
        h1.add_child(b1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        let mut b1 = Button::new_alloc();
        b1.set_text("1 1".into());
        h1.add_child(b1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
        container.add_child(h1.upcast(), false, InternalMode::INTERNAL_MODE_DISABLED);
    }
}

#[godot_api]
impl GodotExt for MainGui {
    fn init(base: Base<Node2D>) -> Self {
        if Engine::singleton().is_editor_hint() {
        } else {
        }

        Self { base }
    }

    fn ready(&mut self) {
        if Engine::singleton().is_editor_hint() {
        } else {
            self.show_sectors();
        }
    }

    fn process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
        } else {
        }
    }
}
