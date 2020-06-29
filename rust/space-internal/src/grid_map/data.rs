use crate::grid::Grid;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum CellKind {
    Free,
    Wall,
}

pub type ShipGrid = Grid<CellKind>;
