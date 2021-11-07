pub const ZERO: V2I = V2I { x: 0, y: 0 };

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, PartialOrd)]
pub struct V2I {
    pub x: i32,
    pub y: i32,
}

impl V2I {
    pub fn new(x: i32, y: i32) -> Self {
        V2I { x, y }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> V2I {
        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        V2I::new(new_x, new_y)
    }

    pub fn as_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl From<(i32, i32)> for V2I {
    fn from((x, y): (i32, i32)) -> Self {
        V2I { x: x, y: y }
    }
}

impl From<[i32; 2]> for V2I {
    fn from(array: [i32; 2]) -> Self {
        V2I {
            x: array[0],
            y: array[1],
        }
    }
}
