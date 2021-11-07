// pub enum Dir {
//     N,
//     S,
//     E,
//     W
// }

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

impl GridCoord {
    pub fn new(x: i32, y: i32) -> Self {
        GridCoord { x, y }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> GridCoord {
        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        GridCoord::new(new_x, new_y)
    }
}

impl From<(u32, u32)> for GridCoord {
    fn from((x, y): (u32, u32)) -> Self {
        GridCoord {
            x: x as i32,
            y: y as i32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub width: u32,
    pub height: u32,
    pub list: Vec<Option<T>>,
}

impl<T> Grid<T> {
    pub fn new(width: u32, height: u32) -> Self {
        let mut list = vec![];
        for _ in 0..width * height {
            list.push(None);
        }

        Grid {
            width,
            height,
            list,
        }
    }

    // TODO: should it exists?
    pub fn set(&mut self, index: u32, value: Option<T>) {
        self.list[index as usize] = value;
    }

    pub fn set_at(&mut self, coord: GridCoord, value: Option<T>) {
        if !self.is_valid_coords(coord) {
            panic!("invalid coords {:?}", coord);
        }
        let index = self.coords_to_index(coord);
        self.list[index as usize] = value;
    }

    // TODO: should it exists?
    pub fn get(&self, index: u32) -> Option<&T> {
        self.list[index as usize].as_ref()
    }

    pub fn get_at(&self, coord: GridCoord) -> Option<&T> {
        if !self.is_valid_coords(coord) {
            panic!("invalid coords {:?}", coord);
        }
        let index = self.coords_to_index(coord);
        self.list[index as usize].as_ref()
    }

    pub fn is_valid_coords(&self, coords: GridCoord) -> bool {
        coords.x >= 0
            && coords.y >= 0
            && coords.x < self.width as i32
            && coords.y < self.height as i32
    }

    // TODO: should return option?
    pub fn coords_to_index(&self, coords: GridCoord) -> usize {
        (coords.y * (self.width as i32) + coords.x) as usize
    }

    pub fn get_neighbours(&self, coords: GridCoord) -> Vec<GridCoord> {
        let mut result = vec![];
        for dy in &[-1, 0, 1] {
            for dx in &[-1, 0, 1] {
                if *dx == 0 && *dy == 0 {
                    continue;
                }

                let new_point = coords.translate(*dx, *dy);

                if !self.is_valid_coords(new_point) {
                    continue;
                }

                result.push(new_point);
            }
        }
        result
    }

    pub fn raytrace(&self, pos: GridCoord, dir_x: i32, dir_y: i32) -> Vec<GridCoord> {
        let mut current = pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = (nx as u32, ny as u32).into();

            if !self.is_valid_coords(current) {
                break;
            }

            match self.get_at(current) {
                Some(_value) => result.push(current),
                None => break,
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_grid_get_neighbors() {
        let grid = Grid::<u32>::new(2, 2);
        let neighbours = grid.get_neighbours(GridCoord::new(0, 0));
        assert_eq!(
            neighbours,
            vec![
                GridCoord::new(1, 0),
                GridCoord::new(0, 1),
                GridCoord::new(1, 1),
            ]
        );
    }

    #[test]
    pub fn test_grid_raytrace() {
        let mut grid = Grid::<u32>::new(4, 2);

        // X###
        // ###
        assert_eq!(grid.raytrace((0, 0).into(), -1, 0), Vec::<GridCoord>::new());

        // #X##
        // ####
        assert_eq!(grid.raytrace((1, 0).into(), -1, 0), Vec::<GridCoord>::new());

        // 0###
        // ####
        grid.set_at((0, 0).into(), Some(0));

        // 0X##
        // ####
        assert_eq!(grid.raytrace((1, 0).into(), -1, 0), vec![(0, 0).into()]);

        // 00##
        // ####
        grid.set_at((1, 0).into(), Some(0));

        // 00X#
        // ####
        assert_eq!(
            grid.raytrace((2, 0).into(), -1, 0),
            vec![(1, 0).into(), (0, 0).into()]
        );

        // 00#X
        // ####
        assert_eq!(grid.raytrace((3, 0).into(), -1, 0), vec![]);

        // X0##
        // ####
        assert_eq!(grid.raytrace((0, 0).into(), 1, 0), vec![(1, 0).into()]);
    }

    #[test]
    pub fn test_grid_group() {
        // TODO
    }
}
