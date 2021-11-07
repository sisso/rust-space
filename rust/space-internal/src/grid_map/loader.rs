use super::data::*;
use crate::grid::{GridCoord};
use std::collections::HashMap;
use std::fmt::Formatter;


pub struct KindCharMap {
    by_char: HashMap<char, CellKind>,
    by_kind: HashMap<CellKind, char>,
}

impl KindCharMap {
    pub fn new(list: &Vec<(char, CellKind)>) -> Self {
        let mut by_char = HashMap::new();
        let mut by_kind = HashMap::new();

        for (chr, kind) in list {
            by_char.insert(*chr, *kind);
            by_kind.insert(*kind, *chr);
        }

        KindCharMap { by_char, by_kind }
    }

    pub fn get_by_char(&self, chr: char) -> Option<CellKind> {
        self.by_char.get(&chr).copied()
    }

    pub fn get_by_kind(&self, kind: CellKind) -> Option<char> {
        self.by_kind.get(&kind).copied()
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
enum LoaderError {
    InvalidChar(char),
}

impl std::error::Error for LoaderError {}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = format!("{:?}", self);
        write!(f, "{}", &value)
    }
}

pub struct Loader;

impl Loader {
    fn load(buffer: &str, character_map: KindCharMap) -> Result<ShipGrid> {
        let mut width = 0;
        let mut height = 0;

        let mut rows = vec![];

        let lines = buffer.split('\n');

        let mut y = 0;
        for line in lines {
            width = width.max(line.len() as u32);
            height += 1;

            let mut x = 0;
            for chr in line.chars() {
                let kind = character_map
                    .get_by_char(chr)
                    .ok_or(LoaderError::InvalidChar(chr))?;

                let entry = (x, y, kind);
                rows.push(entry);

                x += 1;
            }

            y += 1;
        }

        let mut grid = ShipGrid::new(width, height);

        for (x, y, kind) in rows {
            grid.set_at(GridCoord::new(x, y), Some(kind));
        }

        Ok(grid)
    }
}

fn print_shipgrid(_ship_grid: &ShipGrid) {}

#[cfg(test)]
mod test {
    
    use super::*;

    // #[test]
    fn test_0() {
        let characters: KindCharMap =
            KindCharMap::new(&vec![('#', CellKind::Wall), (' ', CellKind::Free)]);

        let buffer = r"###################
#           #     #
#  #####  #####   #
#  #   #      #   #
#  #      # # #   #
#  #####  #####   #
#                 #
###################";

        let grid = Loader::load(buffer, characters);
        assert_eq!("", format!("{:?}", grid));
    }
}
