use godot::engine::{CanvasItem, Node2DVirtual};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct AstroModel {
    pub color: Color,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl AstroModel {}

#[godot_api]
impl Node2DVirtual for AstroModel {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            color: crate::utils::color_white(),
            base,
        }
    }

    fn draw(&mut self) {
        self.base
            .draw_circle(Vector2::new(0.0, 0.0), 1.0, self.color);
    }
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct OrbitModel {
    pub color: Color,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl OrbitModel {}

#[godot_api]
impl Node2DVirtual for OrbitModel {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            color: crate::utils::color_white(),
            base,
        }
    }

    fn draw(&mut self) {
        self.base.draw_arc(
            Vector2::ZERO,
            1.0,
            0.0,
            (360.0f32.to_radians()) as f64,
            128,
            self.color,
            -1.0,
            true,
        );
    }
}
