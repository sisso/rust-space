use glam;

pub type P2 = glam::Vec2;
pub type V2 = glam::Vec2;
pub type V2I = glam::IVec2;
pub type P2I = glam::IVec2;
pub type Affine2 = glam::Affine2;
pub type M4 = glam::Mat4;
pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * std::f32::consts::PI;
pub type Deg = f32;
pub type Rad = f32;
pub type Distance = f32;

pub fn p2v(p2: P2) -> V2 {
    p2
}

pub fn v2p(v2: V2) -> P2 {
    v2
}

pub fn v2(x: f32, y: f32) -> V2 {
    V2::new(x, y)
}

pub fn p2(x: f32, y: f32) -> P2 {
    V2::new(x, y)
}

#[derive(Debug, Clone)]
pub struct Transform2 {
    pos: P2,
    scale: f32,
    angle: f32,
    affine: Affine2,
}

impl Transform2 {
    pub fn new(pos: P2, scale: f32, angle: f32) -> Self {
        Transform2 {
            pos,
            scale,
            angle,
            affine: Affine2::from_scale_angle_translation(V2::ONE * scale, angle, pos),
        }
    }

    pub fn identity() -> Self {
        Transform2 {
            pos: V2::ZERO,
            scale: 1.0,
            angle: 0.0,
            affine: Affine2::IDENTITY,
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
        self.recreate_similarity();
    }

    pub fn set_scale(&mut self, s: f32) {
        self.scale = s;
        self.recreate_similarity();
    }

    pub fn set_angle(&mut self, r: f32) {
        self.angle = r;
        self.recreate_similarity();
    }

    pub fn translate(&mut self, v: V2) {
        self.pos = self.pos + v;
        self.recreate_similarity();
    }

    pub fn scale(&mut self, v: f32) {
        self.scale *= v;
        self.recreate_similarity();
    }

    pub fn get_affine(&self) -> &Affine2 {
        &self.affine
    }

    pub fn point_to_local(&self, p: P2) -> P2 {
        self.affine.transform_point2(p)
    }

    pub fn local_to_point(&self, p: P2) -> P2 {
        self.affine.inverse().transform_point2(p)
    }

    // pub fn get_matrix(&self) -> M4 {
    //     self.affine.into()
    // }

    fn recreate_similarity(&mut self) {
        self.affine =
            Affine2::from_scale_angle_translation(V2::ONE * self.scale, self.angle, self.pos);
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
    use approx::assert_relative_eq;

    #[test]
    fn test_angle_rotation() {
        let deg = 90.0;
        let rad = deg_to_rads(deg);
        let v = rotate_vector_by_angle(P2::new(1.0, 0.0), rad);
        assert_relative_eq!(0.0, v.x);
        assert_eq!(1.0, v.y);
    }

    #[test]
    fn test_transform_2_identity() {
        let t = Transform2::identity();
        let v = V2::new(1.0, 0.0);
        let v2 = t.get_affine().transform_vector2(v);
        assert_eq!(V2::new(1.0, 0.0), v2);

        let p = P2::new(1.0, 0.0);
        let p2 = t.get_affine().transform_point2(p);
        assert_eq!(P2::new(1.0, 0.0), p2);
    }

    #[test]
    fn test_transform_translation() {
        let t = Transform2::new(P2::new(2.0, 1.0), 1.0, 0.0);

        let v = V2::new(1.0, 0.0);
        let v2 = t.get_affine().transform_vector2(v);
        assert_eq!(V2::new(1.0, 0.0), v2);

        let p = P2::new(1.0, 0.0);
        let p2 = t.get_affine().transform_point2(p);
        assert_eq!(P2::new(3.0, 1.0), p2);
    }

    #[test]
    fn test_transform_rotation() {
        let t = Transform2::new(P2::ZERO, 1.0, deg_to_rads(90.0));

        let v = V2::new(1.0, 0.0);
        let v2 = t.get_affine().transform_vector2(v);
        assert_relative_eq!(V2::new(0.0, 1.0), v2);

        let p = P2::new(1.0, 0.0);
        let p2 = t.get_affine().transform_point2(p);
        assert_relative_eq!(P2::new(0.0, 1.0), p2);
    }

    #[test]
    fn test_transform() {
        let t = Transform2::new(P2::new(2.0, 1.0), 2.0, deg_to_rads(90.0));

        let v = V2::new(1.0, 0.0);
        let v2 = t.get_affine().transform_vector2(v);
        assert_relative_eq!(V2::new(0.0, 2.0), v2);

        let p = P2::new(1.0, 0.0);
        let p2 = t.get_affine().transform_point2(p);
        assert_relative_eq!(P2::new(2.0, 3.0), p2);
    }
}

pub fn angle_vector(v: V2) -> f32 {
    v.y.atan2(v.x)
}

pub fn rotate_vector(dir: V2, point: P2) -> P2 {
    let angle = angle_vector(dir);
    rotate_vector_by_angle(point, angle)
}

pub fn rotate_vector_by_angle(point: P2, angle: Rad) -> P2 {
    glam::Mat2::from_angle(angle) * point
}

pub fn deg_to_rads(angle: Deg) -> Rad {
    angle.to_radians()
    // angle * (std::f32::consts::PI / 180.0)
}

pub fn rad_to_deg(rad: Rad) -> Deg {
    // rad * (std::f32::consts::PI * 180.0)
    rad.to_degrees()
}

pub trait IntoP2Ext {
    fn as_p2(self: &Self) -> P2;
}

impl IntoP2Ext for P2I {
    fn as_p2(self: &Self) -> P2 {
        P2::new(self.x as f32, self.y as f32)
    }
}
