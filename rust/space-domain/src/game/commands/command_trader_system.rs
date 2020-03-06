use specs::prelude::*;

pub struct CommandTradeSystem;

impl<'a> System<'a> for CommandTradeSystem {
    type SystemData = (Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
    }
}