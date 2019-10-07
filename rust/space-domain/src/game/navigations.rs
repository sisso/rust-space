use specs::prelude::*;
use specs_derive::*;
use specs::Entity;
use crate::game::sectors::SectorId;
use crate::utils::Position;
use std::collections::VecDeque;
use crate::game::objects::ObjId;

#[derive(Debug, Clone, Component)]
pub enum Navigation {
    MoveTo
}

#[derive(Debug, Clone)]
enum NavigationPlanStep {
    MoveTo { pos: Position, },
    Jump { jump_id: ObjId },
    Dock { target: ObjId },
}


#[derive(Debug, Clone)]
pub struct NavigationPlan {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: Position,
    path: VecDeque<NavigationPlanStep>
}

#[derive(Debug, Clone, Component)]
pub struct NavigationMoveTo {
    target: Entity,
    plan: NavigationPlan
}

//#[derive(SystemData)]
//pub struct UndockMinersData<'a> {
//    entities: Entities<'a>,
//    states: ReadStorage<'a, MineState>,
//    locations: ReadStorage<'a, LocationDock>,
//    has_actions: WriteStorage<'a, HasAction>,
//    undock_actions: WriteStorage<'a, ActionUndock>,
//}
//
//pub struct UndockMinersSystem;
//impl<'a> System<'a> for UndockMinersSystem {
//    type SystemData = UndockMinersData<'a>;
//
//    fn run(&mut self, mut data: UndockMinersData) {
//        use specs::Join;
//
//        let mut to_add = vec![];
//        for (entity, _, _, _) in (&data.entities, &data.states, !&data.has_actions, &data.locations).join() {
//            to_add.push(entity.clone());
//        }
//
//        for entity in to_add {
//            data.undock_actions.insert(entity, ActionUndock);
//        }
//    }
//}

pub struct Navigations {
}

impl Navigations {
    pub fn new() -> Self {
        Navigations {
        }
    }

    pub fn init_world(world: &mut World) {
        world.register::<Navigation>();
        world.register::<NavigationMoveTo>();
    }

    pub fn execute(&mut self, world: &mut World) {
    }
}


