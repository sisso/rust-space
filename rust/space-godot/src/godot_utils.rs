use commons::math::V2;
use godot::prelude::*;

pub fn clear<T>(container: Gd<T>)
where
    T: Inherits<Node>,
{
    let mut container = container.upcast();

    for c in container.get_children().iter_shared() {
        let mut n = c.cast::<Node>();
        container.remove_child(n.clone());
        n.queue_free();
    }
}

#[allow(dead_code)]
pub fn color_black() -> Color {
    Color::from_rgba8(0, 0, 0, 255)
}

#[allow(dead_code)]
pub fn color_blue() -> Color {
    Color::from_rgba8(0, 0, 170, 255)
}

#[allow(dead_code)]
pub fn color_green() -> Color {
    Color::from_rgba8(0, 170, 0, 255)
}

#[allow(dead_code)]
pub fn color_cyan() -> Color {
    Color::from_rgba8(0, 170, 170, 255)
}

#[allow(dead_code)]
pub fn color_red() -> Color {
    Color::from_rgba8(170, 0, 0, 255)
}

#[allow(dead_code)]
pub fn color_magenta() -> Color {
    Color::from_rgba8(170, 0, 170, 255)
}

#[allow(dead_code)]
pub fn color_brown() -> Color {
    Color::from_rgba8(170, 85, 0, 255)
}

#[allow(dead_code)]
pub fn color_light_gray() -> Color {
    Color::from_rgba8(170, 170, 170, 255)
}

#[allow(dead_code)]
pub fn color_dark_gray() -> Color {
    Color::from_rgba8(85, 85, 85, 255)
}

#[allow(dead_code)]
pub fn color_bright_blue() -> Color {
    Color::from_rgba8(85, 85, 255, 255)
}

#[allow(dead_code)]
pub fn color_bright_green() -> Color {
    Color::from_rgba8(85, 255, 85, 255)
}

#[allow(dead_code)]
pub fn color_bright_cyan() -> Color {
    Color::from_rgba8(85, 255, 255, 255)
}

#[allow(dead_code)]
pub fn color_bright_red() -> Color {
    Color::from_rgba8(255, 85, 85, 255)
}

#[allow(dead_code)]
pub fn color_bright_magenta() -> Color {
    Color::from_rgba8(255, 85, 255, 255)
}

#[allow(dead_code)]
pub fn color_yellow() -> Color {
    Color::from_rgba8(255, 255, 85, 255)
}

#[allow(dead_code)]
pub fn color_white() -> Color {
    Color::from_rgba8(255, 255, 255, 255)
}

pub trait V2Vec {
    fn as_vector2(&self) -> Vector2;
}

impl V2Vec for V2 {
    fn as_vector2(&self) -> Vector2 {
        v2vec(self)
    }
}

pub fn v2vec(vec: &V2) -> Vector2 {
    Vector2::new(vec.x, vec.y)
}
