#[derive(Clone,Copy,PartialEq,Debug)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn zero() -> Self {
        V2 { x: 0.0, y: 0.0 }
    }

    pub fn new(x: f32, y: f32) -> Self {
        V2 { x, y }
    }
}

pub type Position = V2;

#[derive(Clone,Copy,Debug)]
pub struct Seconds(pub f32);

impl Seconds {

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

    pub fn next(&mut self) -> u32 {
        let v = self.next;
        self.next += 1;
        v
    }
}
