#[derive(Clone,Copy,PartialEq,Debug)]
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
}

pub type Position = V2;

#[derive(Clone,Copy,Debug)]
pub struct Speed(pub f32);

#[derive(Clone,Copy,Debug)]
pub struct Seconds(pub f32);

impl Seconds {
    pub fn value(&self) -> f32 {
        self.0
    }
}

#[derive(Clone,Copy,Debug)]
pub struct DeltaTime(pub f32);

#[derive(Clone,Copy,Debug)]
pub struct TotalTime(pub f32);

impl TotalTime {
    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn is_after(&self, time: TotalTime) -> bool {
        self.0 >= time.0
    }

    pub fn add(&self, seconds: Seconds) -> TotalTime {
        TotalTime(self.0 + seconds.0)
    }
}

pub struct NextId {
    next: u32,
}

impl NextId {
    pub fn new() -> Self {
        NextId {
            next: 0
        }
    }

    pub fn from(know_max: u32) -> Self {
        NextId {
            next: know_max + 1
        }
    }

    pub fn next(&mut self) -> u32 {
        let v = self.next;
        self.next += 1;
        v
    }
}
