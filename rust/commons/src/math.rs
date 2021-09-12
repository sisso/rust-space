use nalgebra::{Matrix4, Point2, Rotation2, Similarity2, Vector2};

pub type P2 = Point2<f32>;
pub type V2 = Vector2<f32>;
pub type V2I = Vector2<i32>;
pub type Sim2 = Similarity2<f32>;
pub type M4 = Matrix4<f32>;
pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub fn rotate_vector_by_angle(point: P2, angle: f32) -> P2 {
    let rotation = Rotation2::new(angle);
    rotation * point
}

pub fn p2v(p2: P2) -> V2 {
    p2.coords
}

pub fn v2p(v2: V2) -> P2 {
    P2::origin() + v2
}
