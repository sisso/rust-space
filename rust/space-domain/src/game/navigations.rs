use specs::prelude::*;
use specs_derive::*;
use specs::Entity;
use crate::game::sectors::{SectorId, Sectors, JumpId};
use crate::utils::Position;
use std::collections::VecDeque;
use crate::game::objects::ObjId;
use crate::game::actions::*;
use crate::game::navigations::navigation_system::NavigationSystem;
use crate::game::navigations::request_handler_system::NavRequestHandlerSystem;

mod request_handler_system;
mod navigation_system;

///
/// Systems:
/// - set navigation plan from request
/// - execute navigation by create actions
///

#[derive(Debug, Clone, Component, PartialEq)]
pub enum Navigation {
    MoveTo
}

#[derive(Debug, Clone, Component)]
pub enum NavRequest {
    MoveToTarget {
        target: ObjId,
    }
}

#[derive(Debug, Clone)]
pub struct NavigationPlan {
    pub target_sector_id: SectorId,
    pub target_position: Position,
    pub path: VecDeque<Action>
}

#[derive(Debug, Clone, Component)]
pub struct NavigationMoveTo {
    pub target: Entity,
    pub plan: NavigationPlan
}

impl NavigationMoveTo {
    pub fn next(&mut self) -> Option<Action> {
        self.plan.path.pop_front()
    }
}

pub struct Navigations {
}

impl Navigations {
    pub fn new() -> Self {
        Navigations {
        }
    }

    pub fn init_world(world: &mut World, dispatcher: &mut DispatcherBuilder) {
        dispatcher.add(NavRequestHandlerSystem, "navigation_request_handler", &[]);
        dispatcher.add(NavigationSystem, "navigation", &["navigation_request_handler"]);
    }

    pub fn execute(&mut self, world: &mut World) {
    }

    pub fn create_plan(sectors: &Sectors, from_sector_id: SectorId, from_pos: Position, to_sector_id: SectorId, to_pos: Position, is_docked: bool) -> NavigationPlan {
        let safe = 100;
        let mut path = VecDeque::new();

        if is_docked {
            path.push_back(Action::Undock);
        }

        let mut current_pos = from_pos;
        let mut current_sector = from_sector_id;

        for i in 0..safe {
            if i + 1 == safe {
                panic!();
            }

            if current_sector == to_sector_id {
                path.push_back(Action::MoveTo { pos: to_pos });
                break;
            } else {
                let jump = sectors.find_jump(current_sector, to_sector_id).unwrap();

                path.push_back(Action::MoveTo { pos: jump.pos });
                path.push_back(Action::Jump { jump_id: jump.id });

                current_sector = jump.to_sector_id;
                current_pos = jump.to_pos;
            }
        }

        info!(target: "create_plan", "navigation path from {:?}/{:?} to {:?}/{:?}: {:?}",
            from_sector_id, from_pos, to_sector_id, to_pos, path);

        NavigationPlan {
            target_sector_id: to_sector_id,
            target_position: to_pos,
            path,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;

    #[test]
    fn create_plan() {
        let sectors = new_test_sectors();

        let plan = Navigations::create_plan(
            &sectors,
            SECTOR_0,
            Position::new(10.0, 0.0),
            SECTOR_1,
            Position::new(0.0, 10.0),
            true
        );

        assert_eq!(plan.target_sector_id, SECTOR_1);
        assert_eq!(plan.target_position, Position::new(0.0, 10.0));
        assert_eq!(plan.path.len(), 4);

        match plan.path.get(0).unwrap() {
            Action::Undock => {},
            other => panic!(),
        }

        match plan.path.get(1).unwrap() {
            Action::MoveTo { pos } => assert_eq!(pos.clone(), JUMP_0_TO_1.pos),
            other => panic!(),
        }

        match plan.path.get(2).unwrap() {
            Action::Jump { jump_id } => assert_eq!(jump_id.clone(), JUMP_0_TO_1.id),
            other => panic!(),
        }

        match plan.path.get(3).unwrap() {
            Action::MoveTo { pos } => assert_eq!(pos.clone(), Position::new(0.0, 10.0)),
            other => panic!(),
        }
    }
}