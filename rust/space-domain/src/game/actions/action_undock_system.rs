use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::actions::*;

pub struct UndockSystem;

#[derive(SystemData)]
pub struct UndockData<'a> {
    entities: Entities<'a>,
}

impl<'a> System<'a> for UndockSystem {
    type SystemData = UndockData<'a>;

    fn run(&mut self, mut data: UndockData) {
        for _ in (&*data.entities).join() {
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stuff() {
    }
}
