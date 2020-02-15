use specs::prelude::*;
use crate::game::extractables::Extractable;
use crate::game::wares::Cargo;
use crate::utils::TotalTime;
use crate::game::actions::{ActionExtract, ActionProgress};

pub struct ActionExtractSystem;

#[derive(SystemData)]
pub struct ActionExtractData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    extractables: ReadStorage<'a, Extractable>,
    cargo: ReadStorage<'a, Cargo>,
    action: ReadStorage<'a, ActionExtract>,
    action_progress: ReadStorage<'a, ActionProgress>,
}

impl<'a> System<'a> for ActionExtractSystem {
   type SystemData = ActionExtractData<'a>;

    fn run(&mut self, mut data: ActionExtractData) {
       trace!("running");

        let now = data.total_time.clone();

        for (
            entity,
            action,
            cargo,
            _
        ) in (
            &*data.entities,
            &data.action,
            &data.cargo,
            !&data.action_progress,
        ).join() {

        }
    }
}
