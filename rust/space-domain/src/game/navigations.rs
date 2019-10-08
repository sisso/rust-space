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

/// create navigation plans for new miners
///
///
//pub struct CreateNavigationSystem;
//
//#[derive(SystemData)]
//pub struct CreateNavigationData<'a> {
//    entities: Entities<'a>,
//    sectors_index: Read<'a, SectorsIndex>,
//    commands_mine: ReadStorage<'a, CommandMine>,
//    actions_mine: ReadStorage<'a, ActionMine>,
//    navigations: WriteStorage<'a, Navigation>,
//    navigations_move_to: WriteStorage<'a, NavigationMoveTo>,
//}
//
//impl<'a> System<'a> for CreateNavigationSystem {
//    type SystemData = CreateNavigationData<'a>;
//
//    fn run(&mut self, mut data: CreateNavigationData) {
//        use specs::Join;
//
//        let sector_index = data.sectors_index.borrow();
//
//
//        for (commands_mine) in (&data.commands_mine, !&data.navigations, !&data.actions_mine).join() {
//
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


