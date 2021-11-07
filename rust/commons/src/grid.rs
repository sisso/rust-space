use super::v2i::V2I;
use crate::recti::RectI;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

pub const DIR_ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];
pub type Coord = V2I;
pub type Index = i32;

/**
    0 1 2
    3 4 5
    6 7 8
*/
#[derive(Debug, Clone)]
pub struct Grid<T: Default> {
    pub width: i32,
    pub height: i32,
    pub list: Vec<T>,
}

impl<T: Default> Grid<T> {
    pub fn new_square(size: i32) -> Self {
        Grid::new(size, size)
    }

    pub fn new(width: i32, height: i32) -> Self {
        let mut list = vec![];
        for _ in 0..width * height {
            list.push(Default::default());
        }

        Grid {
            width,
            height,
            list,
        }
    }

    pub fn set(&mut self, index: Index, value: T) {
        assert!(self.is_valid_index(index));
        self.list[index as usize] = value;
    }

    pub fn set_at(&mut self, coord: &Coord, value: T) -> T {
        assert!(self.is_valid_coords(coord));
        let index = self.coords_to_index(coord);
        std::mem::replace(&mut self.list[index as usize], value)
    }

    pub fn get(&self, index: i32) -> &T {
        assert!(self.is_valid_index(index));
        &self.list[index as usize]
    }

    pub fn get_at(&self, coord: &Coord) -> &T {
        assert!(self.is_valid_coords(&coord));
        let index = self.coords_to_index(coord);
        &self.list[index as usize]
    }

    pub fn get_at_opt(&self, coord: &Coord) -> Option<&T> {
        let index = self.coords_to_index(coord);
        if self.is_valid_coords(coord) {
            Some(&self.list[index as usize])
        } else {
            None
        }
    }

    // not safe to use if you try to verify an X axis beyond grid bounds
    pub fn is_valid_index(&self, index: Index) -> bool {
        index < self.list.len() as i32
    }

    pub fn is_valid_coords(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.width as i32 && coord.y < self.height as i32
    }

    pub fn coords_to_index(&self, coords: &Coord) -> Index {
        coords_to_index(self.width, coords)
    }

    pub fn get_valid_4_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_4_neighbours(coords)
            .into_iter()
            .map(|(_, i)| i)
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn get_valid_8_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_8_neighbours(coords)
            .into_iter()
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn raytrace(&self, pos: &Coord, dir_x: i32, dir_y: i32) -> Vec<Coord> {
        let mut current = *pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = V2I::new(nx, ny);

            if !self.is_valid_coords(&current) {
                break;
            }

            result.push(current);
        }

        result
    }
}

pub fn coords_to_index(width: i32, xy: &Coord) -> Index {
    xy.y * width + xy.x
}

pub fn index_to_coord(_width: Index) -> Coord {
    todo!()
}

/// return sequentially with DIR_ALL
pub fn get_4_neighbours(coords: &Coord) -> Vec<(Dir, Coord)> {
    vec![
        (Dir::N, coords.translate(0, -1)),
        (Dir::E, coords.translate(1, 0)),
        (Dir::S, coords.translate(0, 1)),
        (Dir::W, coords.translate(-1, 0)),
    ]
}

pub fn get_8_neighbours(coords: &Coord) -> Vec<Coord> {
    vec![
        coords.translate(0, -1),
        coords.translate(1, -1),
        coords.translate(1, 0),
        coords.translate(1, 1),
        coords.translate(0, 1),
        coords.translate(-1, 1),
        coords.translate(-1, 0),
        coords.translate(-1, -1),
    ]
}

#[derive(Debug, Clone)]
pub struct FlexGrid<T> {
    pub cells: HashMap<Coord, T>,
}

impl<T> FlexGrid<T> {
    pub fn new() -> Self {
        FlexGrid {
            cells: HashMap::new(),
        }
    }

    pub fn set_at(&mut self, coord: &Coord, value: Option<T>) {
        match value {
            Some(v) => self.cells.insert(coord.to_owned(), v),
            None => self.cells.remove(coord),
        };
    }

    pub fn get_at(&self, coord: &Coord) -> Option<&T> {
        self.cells.get(coord)
    }
}

#[derive(Clone, Debug)]
pub struct PGrid<T: Default> {
    pub rect: RectI,
    pub grid: Grid<T>,
}

impl<T: Default> PGrid<T> {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        let rect = RectI::new(x, y, width, height);

        PGrid {
            rect: rect,
            grid: Grid::new(width, height),
        }
    }

    pub fn from_grid(coord: &V2I, grid: Grid<T>) -> Self {
        let rect = RectI::new(coord.x, coord.y, grid.width, grid.height);

        PGrid {
            rect: rect,
            grid: grid,
        }
    }

    pub fn get_pos(&self) -> Coord {
        self.rect.get_top_left().clone()
    }

    pub fn set_pos(&mut self, pos: &V2I) {
        self.rect = self.rect.copy_with_pos(pos);
    }

    pub fn get_width(&self) -> i32 {
        self.grid.width
    }

    pub fn get_height(&self) -> i32 {
        self.grid.height
    }

    pub fn to_local(&self, coord: &Coord) -> Coord {
        self.rect.to_local(coord)
    }

    pub fn to_global(&self, coord: &Coord) -> Coord {
        self.rect.to_global(coord)
    }

    pub fn set_at(&mut self, coord: &Coord, value: T) -> T {
        let local = self.to_local(coord);
        assert!(self.grid.is_valid_coords(&local));
        self.grid.set_at(&local, value)
    }

    pub fn get_at(&self, coord: &Coord) -> &T {
        let local = self.to_local(coord);
        assert!(self.grid.is_valid_coords(&local));
        self.grid.get_at(&local)
    }

    pub fn get_at_opt(&self, coord: &Coord) -> Option<&T> {
        let local = self.to_local(coord);
        self.grid.get_at_opt(&local)
    }

    pub fn is_valid_coords(&self, coord: &Coord) -> bool {
        self.rect.is_inside(coord)
    }

    pub fn get_valid_4_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_4_neighbours(coords)
            .into_iter()
            .map(|(_, i)| i)
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn get_valid_8_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_8_neighbours(coords)
            .into_iter()
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn raytrace(&self, pos: &Coord, dir_x: i32, dir_y: i32) -> Vec<Coord> {
        let mut current = *pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = V2I::new(nx, ny);

            if !self.is_valid_coords(&current) {
                break;
            }

            result.push(current);
        }

        result
    }
}

impl<T: Default> From<PGrid<T>> for Grid<T> {
    fn from(pgrid: PGrid<T>) -> Self {
        pgrid.grid
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_grid_get_neighbors() {
        let neighbours = get_4_neighbours(&Coord::new(0, 0));
        assert_eq!(
            neighbours,
            vec![
                (Dir::N, Coord::new(0, -1)),
                (Dir::E, Coord::new(1, 0)),
                (Dir::S, Coord::new(0, 1)),
                (Dir::W, Coord::new(-1, 0)),
            ]
        );
    }

    #[test]
    pub fn test_grid_get_valid_neighbors() {
        let grid = Grid::<i32>::new(2, 2);
        let neighbours = grid.get_valid_8_neighbours(&Coord::new(0, 0));
        assert_eq!(
            vec![Coord::new(1, 0), Coord::new(1, 1), Coord::new(0, 1),],
            neighbours
        );
    }

    #[test]
    pub fn test_grid_raytrace() {
        let mut grid = Grid::<i32>::new(4, 2);

        // X###
        // ###
        assert_eq!(grid.raytrace(&(0, 0).into(), -1, 0), Vec::<Coord>::new());

        // #X##
        // ####
        assert_eq!(grid.raytrace(&(1, 0).into(), -1, 0), Vec::<Coord>::new());

        // 0###
        // ####
        grid.set_at(&(0, 0).into(), 0);

        // 0X##
        // ####
        assert_eq!(grid.raytrace(&(1, 0).into(), -1, 0), vec![(0, 0).into()]);

        // 00##
        // ####
        grid.set_at(&(1, 0).into(), 0);

        // 00X#
        // ####
        assert_eq!(
            grid.raytrace(&(2, 0).into(), -1, 0),
            vec![(1, 0).into(), (0, 0).into()]
        );

        // 00#X
        // ####
        assert_eq!(grid.raytrace(&(3, 0).into(), -1, 0), vec![]);

        // X0##
        // ####
        assert_eq!(grid.raytrace(&(0, 0).into(), 1, 0), vec![(1, 0).into()]);
    }
}
