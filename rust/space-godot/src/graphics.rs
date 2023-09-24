use crate::utils;
use godot::engine::Node2DVirtual;
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
    #[must_use]
    pub fn new_alloc() -> Gd<Self> {
        unsafe {
            let __class_name = StringName::from("AstroModel");
            let __object_ptr =
                sys::interface_fn!(classdb_construct_object)(__class_name.string_sys());
            Gd::from_obj_sys(__object_ptr)
        }
    }

    #[func]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

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
    pub base: Base<Node2D>,
}

#[godot_api]
impl OrbitModel {
    #[func]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    #[must_use]
    pub fn new_alloc() -> Gd<Self> {
        unsafe {
            let __class_name = StringName::from("OrbitModel");
            let __object_ptr =
                sys::interface_fn!(classdb_construct_object)(__class_name.string_sys());
            Gd::from_obj_sys(__object_ptr)
        }
    }
}

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

    #[must_use]
    pub fn new_alloc() -> Gd<Self> {
        unsafe {
            let __class_name = StringName::from("SelectedModel");
            let __object_ptr =
                sys::interface_fn!(classdb_construct_object)(__class_name.string_sys());
            Gd::from_obj_sys(__object_ptr)
        }
    }
}

#[godot_api]
impl Node2DVirtual for SelectedModel {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            color: utils::color_white(),
            base,
        }
    }

    fn draw(&mut self) {
        self.base.draw_rect(
            Rect2::new(Vector2::new(-1.0, -1.0), Vector2::new(2.0, 2.0)),
            self.color,
        );
    }
}
