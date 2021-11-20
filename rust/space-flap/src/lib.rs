#![allow(unused)]

#[derive(Clone)]
pub struct SectorData {
    index: i32,
    coords: (f32, f32),
}

impl SectorData {
    pub fn new() -> Self {
        SectorData {
            index: 0,
            coords: (0.0, 0.0),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }
    pub fn coords(&self) -> (f32, f32) {
        self.coords.clone()
    }
}

pub struct SpaceGame {
    sectors: Vec<SectorData>,
}

impl SpaceGame {
    pub fn new(args: Vec<String>) -> Self {
        SpaceGame { sectors: vec![] }
    }

    pub fn get_sectors_len(&self) -> i32 {
        self.sectors.len() as i32
    }

    pub fn get_sector(&self, index: i32) -> SectorData {
        self.sectors
            .get(index as usize)
            .cloned()
            .expect("invalid sector index")
    }
}

include!(concat!(env!("OUT_DIR"), "/glue.rs"));

#[cfg(test)]
mod test {
    #[test]
    fn test1() {}
}
