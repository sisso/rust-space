use crate::game::actions::*;

use crate::game::navigations::navigation_request_handler_system::NavRequestHandlerSystem;
use crate::game::navigations::navigation_system::NavigationSystem;
use crate::game::objects::ObjId;
use crate::game::sectors::{Jump, Sector, SectorId};
use crate::game::{GameInitContext, RequireInitializer};

use specs::prelude::*;

use crate::game::locations::Location;
use commons::math::P2;
use std::collections::VecDeque;

mod navigation_request_handler_system;
mod navigation_system;

///
/// Systems:
/// - set navigation plan from request
/// - execute navigation by create actions
///

#[derive(Debug, Clone, Component, PartialEq)]
pub enum Navigation {
    MoveTo,
}

#[derive(Debug, Clone, Component)]
pub enum NavRequest {
    MoveToTarget { target_id: ObjId },
    MoveAndDockAt { target_id: ObjId },
}

#[derive(Debug, Clone)]
pub enum Target {
    Pos(P2),
    ObjPos(ObjId),
}

#[derive(Debug, Clone)]
pub struct NavigationPlan {
    pub path: VecDeque<Action>,
}

impl NavigationPlan {
    pub fn append_dock(&mut self, target_id: ObjId) {
        self.path.push_back(Action::Dock { target_id });
    }
}

#[derive(Debug, Clone, Component)]
pub struct NavigationMoveTo {
    pub target_id: Entity,
    pub plan: NavigationPlan,
}

impl NavigationMoveTo {
    pub fn next(&mut self) -> Option<Action> {
        self.plan.path.pop_front()
    }
}

pub struct Navigations;

impl RequireInitializer for Navigations {
    fn init(context: &mut GameInitContext) {
        context
            .dispatcher
            .add(NavRequestHandlerSystem, "navigation_request_handler", &[]);

        context.dispatcher.add(
            NavigationSystem,
            "navigation",
            &["navigation_request_handler"],
        );
    }
}

pub fn create_plan<'a>(
    entities: &Entities<'a>,
    sectors: &ReadStorage<'a, Sector>,
    jumps: &ReadStorage<'a, Jump>,
    locations: &ReadStorage<'a, Location>,
    from_sector_id: SectorId,
    _from_pos: P2,
    to_sector_id: SectorId,
    target: Target,
    is_docked: bool,
) -> NavigationPlan {
    let mut path = VecDeque::new();
    if is_docked {
        path.push_back(Action::Undock);
    }

    let sector_path = match super::sectors::find_path(
        entities,
        sectors,
        jumps,
        locations,
        from_sector_id,
        to_sector_id,
    ) {
        Some(path) => path,
        None => panic!(
            "fail to find path from sectors {:?} to {:?}",
            from_sector_id, to_sector_id
        ),
    };

    for leg in &sector_path {
        path.push_back(Action::MoveToTargetPos {
            target_id: leg.jump_id,
            last_position: Some(leg.jump_pos),
        });
        path.push_back(Action::Jump {
            jump_id: leg.jump_id,
        });
    }

    match target {
        Target::Pos(pos) => path.push_back(Action::MoveTo { pos: pos }),
        Target::ObjPos(id) => path.push_back(Action::MoveToTargetPos {
            target_id: id,
            last_position: None,
        }),
    }

    return NavigationPlan { path };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::navigations;
    use crate::game::sectors::test_scenery::*;
    use crate::utils::Position;

    #[test]
    fn create_plant_test() {
        let mut world = World::new();

        let sector_scenery = setup_sector_scenery(&mut world);

        let entities = world.entities();
        let sectors = world.read_storage::<Sector>();
        let jumps = world.read_storage::<Jump>();
        let locations = world.read_storage::<Location>();

        let plan = navigations::create_plan(
            &entities,
            &sectors,
            &jumps,
            &locations,
            sector_scenery.sector_0,
            P2::ZERO,
            sector_scenery.sector_1,
            Target::Pos(Position::new(0.0, 10.0)),
            true,
        );

        assert_eq!(plan.path.len(), 4);

        match plan.path.get(0).unwrap() {
            Action::Undock => {}
            _other => panic!(),
        }

        match plan.path.get(1).unwrap() {
            Action::MoveToTargetPos { last_position, .. } => {
                assert_eq!(last_position.clone(), Some(sector_scenery.jump_0_to_1_pos))
            }
            other => panic!("found {:?}", other),
        }

        match plan.path.get(2).unwrap() {
            Action::Jump { jump_id } => assert_eq!(jump_id.clone(), sector_scenery.jump_0_to_1),
            _other => panic!(),
        }

        match plan.path.get(3).unwrap() {
            Action::MoveTo { pos } => assert_eq!(pos.clone(), Position::new(0.0, 10.0)),
            _other => panic!(),
        }
    }
}
