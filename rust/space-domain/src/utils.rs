use serde::{Deserialize, Serialize};
use std::time::Duration;
use specs::Entity;

pub const MIN_DISTANCE: f32 = 0.01;
pub const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

const V2_ZERO: V2 = V2 { x: 0.0, y: 0.0 };

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        V2 { x, y }
    }

    pub fn add(&self, other: &V2) -> V2 {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(&self, other: &V2) -> V2 {
        V2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn normalized(&self) -> V2 {
        self.mult(1.0 / self.length_sqr().sqrt())
    }

    pub fn length(&self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn length_sqr(&self) -> f32 {
        (self.x * self.x) + (self.y * self.y)
    }

    pub fn mult(&self, scale: f32) -> V2 {
        V2 {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    pub fn div(&self, scale: f32) -> V2 {
        self.mult(1.0 / scale)
    }

    pub fn distance(from: &V2, to: &V2) -> f32 {
        to.sub(from).length()
    }

    /// returns the new position and true if hit the target
    pub fn move_towards(from: V2, to: V2, max_distance: f32) -> (V2, bool) {
        let delta = to.sub(&from);
        // delta == zero can cause length sqr NaN
        let length_sqr = delta.length_sqr();
        if length_sqr.is_nan() || length_sqr <= max_distance {
            (to, true)
        } else {
            let norm = delta.div(length_sqr.sqrt());
            let mov = norm.mult(max_distance);
            let new_position = from.add(&mov);
            (new_position, false)
        }
    }
}

pub type Position = V2;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Speed(pub f32);

impl Speed {
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

pub type Seconds = DeltaTime;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        DeltaTime(0.0)
    }
}

impl DeltaTime {
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TotalTime(pub f64);

impl Default for TotalTime {
    fn default() -> Self {
        TotalTime(0.0)
    }
}

impl TotalTime {
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn is_after(&self, time: TotalTime) -> bool {
        self.0 >= time.0
    }

    pub fn is_before(&self, time: TotalTime) -> bool {
        self.0 <= time.0
    }

    pub fn add(&self, delta: DeltaTime) -> TotalTime {
        TotalTime(self.0 + delta.0 as f64)
    }

    pub fn sub(&self, other: TotalTime) -> DeltaTime {
        DeltaTime((self.0 - other.0) as f32)
    }

    /// convert to milliseconds
    pub fn as_u64(&self) -> u64 {
        (self.0 * 1000.0) as u64
    }
}

pub struct NextId {
    next: u32,
}

impl NextId {
    pub fn new() -> Self {
        NextId { next: 0 }
    }

    pub fn from(know_max: u32) -> Self {
        NextId { next: know_max + 1 }
    }

    pub fn next(&mut self) -> u32 {
        let v = self.next;
        self.next += 1;
        v
    }
}

pub trait IdAsU32Support {
    fn as_u32(&self) -> u32;
}

impl IdAsU32Support for Entity {
    fn as_u32(&self) -> u32 {
        self.id()
    }
}


#[test]
fn test_total_time_give_us_hundred_years_game_60_fps_precision() {
    let total = TotalTime(100.0 * 360.0 * 24.0 * 60.0 * 60.0);
    let change = DeltaTime(1.0 / 60.0);
    let new_total = total.add(change.into());
    let diff = new_total.sub(total);
    let diff_expected = (change.0 - diff.0).abs();
    println!("{:?}", total);
    println!("{:?}", change);
    println!("{:?}", new_total);
    println!("{:?}", diff);
    assert!(diff_expected < 0.0166666);
}
