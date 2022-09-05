use nalgebra::{Matrix4, Point2, Rotation2, Similarity2, Similarity3, Vector2, Vector3};

pub type P2 = Point2<f32>;
pub type V2 = Vector2<f32>;
pub type V2I = Vector2<i32>;
pub type P2I = Point2<i32>;
pub type Sim2 = Similarity2<f32>;
pub type M4 = Matrix4<f32>;
pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub fn p2v(p2: P2) -> V2 {
    p2.coords
}

pub fn v2p(v2: V2) -> P2 {
    P2::origin() + v2
}

pub fn v2(x: f32, y: f32) -> V2 {
    Vector2::new(x, y)
}

pub fn p2(x: f32, y: f32) -> P2 {
    Point2::new(x, y)
}

#[derive(Debug, Clone)]
pub struct Transform2 {
    pos: P2,
    scale: f32,
    angle: f32,
    similarity: Similarity2<f32>,
}

impl Transform2 {
    pub fn new(pos: P2, scale: f32, rotation: f32) -> Self {
        Transform2 {
            pos,
            scale,
            angle: rotation,
            similarity: Similarity2::new(pos.coords.clone(), rotation, scale),
        }
    }

    pub fn identity() -> Self {
        Transform2 {
            pos: Point2::origin(),
            scale: 1.0,
            angle: 0.0,
            similarity: Similarity2::identity(),
        }
    }

    pub fn get_pos(&self) -> &P2 {
        &self.pos
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_pos(&mut self, p: P2) {
        self.pos = p;
        self.recriate_similarity();
    }

    pub fn set_scale(&mut self, s: f32) {
        self.scale = s;
        self.recriate_similarity();
    }

    pub fn set_angle(&mut self, r: f32) {
        self.angle = r;
        self.recriate_similarity();
    }

    pub fn translate(&mut self, v: V2) {
        self.pos = self.pos + v;
        self.recriate_similarity();
    }

    pub fn scale(&mut self, v: f32) {
        self.scale *= v;
        self.recriate_similarity();
    }

    pub fn get_similarity(&self) -> &Similarity2<f32> {
        &self.similarity
    }

    pub fn point_to_local(&self, p: &P2) -> P2 {
        self.similarity.transform_point(&p)
    }

    pub fn local_to_point(&self, p: &P2) -> P2 {
        self.similarity.inverse_transform_point(&p)
    }

    pub fn get_matrix(&self) -> M4 {
        let sim = Similarity3::new(
            Vector3::new(self.pos.coords.x, self.pos.coords.y, 0.0),
            Vector3::new(0.0, 0.0, self.angle),
            self.scale,
        );

        sim.into()
    }

    fn recriate_similarity(&mut self) {
        self.similarity = Similarity2::new(self.pos.coords.clone(), self.angle, self.scale);
    }
}

/// returns the value between v0 and v1 on t
pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    v0 + clamp01(t) * (v1 - v0)
}

/// returns % of t between v0 and v1
pub fn inverse_lerp(v0: f32, v1: f32, t: f32) -> f32 {
    if v0 == v1 {
        0.0
    } else {
        clamp01((t - v0) / (v1 - v0))
    }
}

///
/// Lerp between v0 and v1 giving the value of t between t0 and t1
///
/// t <= t0, returns v0
/// t >= t1, returns v1
///
/// TODO: use map_value where t (change arguments order)
#[deprecated()]
pub fn lerp_2(v0: f32, v1: f32, t0: f32, t1: f32, t: f32) -> f32 {
    let tt = inverse_lerp(t0, t1, t);
    lerp(v0, v1, tt)
}

/// Lerp the value t between t0 and t1 into v0 and v1
pub fn map_value(t: f32, t0: f32, t1: f32, v0: f32, v1: f32) -> f32 {
    let tt = inverse_lerp(t0, t1, t);
    lerp(v0, v1, tt)
}

pub fn clamp01(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lerp_2() {
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 1.0, 0.5), 0.5);
        assert_eq!(lerp_2(0.0, 2.0, 0.0, 1.0, 0.5), 1.0);
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 2.0, 1.0), 0.5);
    }
}

pub fn angle_vector(v: V2) -> f32 {
    v.y.atan2(v.x)
}

// TODO: remove?
pub fn rotate_vector(dir: V2, point: P2) -> P2 {
    let angle = angle_vector(dir);
    rotate_vector_by_angle(point, angle)
}

// TODO: remove?
pub fn rotate_vector_by_angle(point: P2, angle: f32) -> P2 {
    let rotation = Rotation2::new(angle);
    rotation * point
}
