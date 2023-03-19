use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;

struct SpaceGame;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceGame {}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Stuff {
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl GodotExt for Stuff {
    fn init(base: Base<Node2D>) -> Self {
        Stuff { base: base }
    }

    fn ready(&mut self) {
        godot_print!("ready 4");
    }

    fn process(&mut self, delta: f64) {
        // godot_print!("update for {delta}");
        // self.translate(Vector2::new(delta as f32, 0.0));
    }
}
