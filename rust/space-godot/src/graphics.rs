use crate::utils;
use godot::prelude::*;
use godot::sys;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct AstroModel {
    pub color: Color,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl AstroModel {
    #[func]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

#[godot_api]
impl INode2D for AstroModel {
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
    pub base: Base<Node2D>,
}

#[godot_api]
impl OrbitModel {
    #[func]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

#[godot_api]
impl INode2D for OrbitModel {
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
            360.0f32.to_radians(),
            32,
            self.color,
        );
    }
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct SelectedModel {
    pub color: Color,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl SelectedModel {
    #[func]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

#[godot_api]
impl INode2D for SelectedModel {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            color: utils::color_white(),
            base,
        }
    }

    fn draw(&mut self) {
        self.base
            .draw_rect_ex(
                Rect2::new(Vector2::new(-1.0, -1.0), Vector2::new(2.0, 2.0)),
                self.color,
            )
            .filled(false)
            .done();
    }
}
