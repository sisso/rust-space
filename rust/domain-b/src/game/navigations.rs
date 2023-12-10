use crate::game::actions::*;

use crate::game::objects::ObjId;
use crate::game::sectors::{FindPathParams, Jump, Sector, SectorId};

use bevy_ecs::prelude::*;

use crate::game::locations::{LocationDocked, LocationOrbit, LocationSpace, Locations};
use crate::game::sectors;
use commons::math::P2;
use std::collections::VecDeque;

mod navigation_request_handler_system;
mod navigation_system;

///
/// Systems:
/// - set navigation plan from request
/// - execute navigation by create actions
///

#[derive(Debug, Clone, Component)]
pub struct Navigation {
    pub request: NavRequest,
    pub plan: NavigationPlan,
}

#[derive(Debug, Clone, Component, PartialEq)]
pub enum NavRequest {
    OrbitTarget { target_id: ObjId },
    MoveToTarget { target_id: ObjId },
    MoveAndDockAt { target_id: ObjId },
    MoveToPos { sector_id: SectorId, pos: P2 },
}

#[derive(Debug, Clone)]
pub enum PlanTarget {
    Pos(P2),
    ObjPos(ObjId),
    DockAt(ObjId),
    Orbit(ObjId),
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

impl Navigation {
    pub fn take_next(&mut self) -> Option<Action> {
        self.plan.path.pop_front()
    }
}

pub struct Navigations;

pub fn create_plan_system(
    In((obj_id, request)): In<(Entity, NavRequest)>,
    query_entity: Query<(Option<&LocationDocked>, Option<&LocationOrbit>)>,
    query_locations: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    query_sectors: Query<&Sector>,
    query_jumps: Query<(&Jump, &LocationSpace)>,
) -> Result<NavigationPlan, &'static str> {
    create_plan(
        &query_entity,
        &query_locations,
        &query_sectors,
        &query_jumps,
        obj_id,
        &request,
    )
}

pub fn create_plan(
    query_entity: &Query<(Option<&LocationDocked>, Option<&LocationOrbit>)>,
    query_locations: &Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    query_sectors: &Query<&Sector>,
    query_jumps: &Query<(&Jump, &LocationSpace)>,
    obj_id: Entity,
    request: &NavRequest,
) -> Result<NavigationPlan, &'static str> {
    let mut path = VecDeque::new();

    let (maybe_docked, maybe_orbiting) =
        query_entity.get(obj_id).map_err(|_| "obj_id not found")?;
    if maybe_docked.is_some() {
        path.push_back(Action::Undock);
    }
    if maybe_orbiting.is_some() {
        path.push_back(Action::Deorbit);
    }

    let from_location = Locations::resolve_space_position(&query_locations, obj_id)
        .ok_or("provided obj has no location")?;

    let to_location = match request {
        NavRequest::OrbitTarget { target_id } => {
            Locations::resolve_space_position(&query_locations, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveToTarget { target_id } => {
            Locations::resolve_space_position(&query_locations, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveAndDockAt { target_id } => {
            Locations::resolve_space_position(&query_locations, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveToPos { sector_id, pos } => LocationSpace {
            sector_id: *sector_id,
            pos: *pos,
        },
    };

    let sector_path = sectors::find_path_raw(
        query_sectors,
        query_jumps,
        FindPathParams::new(from_location.sector_id, to_location.sector_id),
    )
    .ok_or("fail to find jump path between sectors")?;

    for leg in &sector_path {
        path.push_back(Action::MoveToTargetPos {
            target_id: leg.jump_id,
            last_position: Some(leg.jump_pos),
        });
        path.push_back(Action::Jump {
            jump_id: leg.jump_id,
        });
    }

    match request {
        NavRequest::MoveToTarget { target_id } => path.push_back(Action::MoveToTargetPos {
            target_id: *target_id,
            last_position: None,
        }),
        NavRequest::MoveToPos { pos, .. } => path.push_back(Action::MoveTo { pos: *pos }),
        NavRequest::MoveAndDockAt { target_id } => {
            path.push_back(Action::MoveToTargetPos {
                target_id: *target_id,
                last_position: None,
            });
            path.push_back(Action::Dock {
                target_id: *target_id,
            });
        }
        NavRequest::OrbitTarget { target_id } => {
            path.push_back(Action::MoveToTargetPos {
                target_id: *target_id,
                last_position: None,
            });
            path.push_back(Action::Orbit {
                target_id: *target_id,
            });
        }
    }

    return Ok(NavigationPlan { path });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;
    use crate::game::utils::Speed;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn create_plant_test_from_docked_into_other_dock() {
        let mut world = World::new();

        let scn = setup_sector_scenery(&mut world);

        let starting_station_id = world
            .spawn_empty()
            .insert(LocationSpace {
                sector_id: scn.sector_0,
                pos: P2::new(0.0, 0.0),
            })
            .id();

        let target_station_id = world
            .spawn_empty()
            .insert(LocationSpace {
                sector_id: scn.sector_1,
                pos: P2::new(1.0, 0.0),
            })
            .id();

        let fleet_id = world
            .spawn_empty()
            .insert(LocationDocked {
                parent_id: starting_station_id,
            })
            .id();

        let plan = world
            .run_system_once_with(
                (
                    fleet_id,
                    NavRequest::MoveAndDockAt {
                        target_id: target_station_id,
                    },
                ),
                create_plan_system,
            )
            .expect("fail to generate plan");

        assert_eq!(plan.path.len(), 5);
        match &plan.path[0] {
            Action::Undock => {}
            other => panic!("unexpected action {:?}", other),
        }
        match &plan.path[1] {
            Action::MoveToTargetPos { last_position, .. } => {
                assert_eq!(last_position.clone(), Some(scn.jump_0_to_1_pos))
            }
            other => panic!("found {:?}", other),
        }
        match &plan.path[2] {
            Action::Jump { jump_id } => assert_eq!(jump_id.clone(), scn.jump_0_to_1),
            other => panic!("unexpected action {:?}", other),
        }
        match &plan.path[3] {
            Action::MoveToTargetPos { target_id, .. } => assert_eq!(target_station_id, *target_id),
            other => panic!("unexpected action {:?}", other),
        }
        match &plan.path[4] {
            Action::Dock { target_id } => assert_eq!(target_station_id, *target_id),
            other => panic!("unexpected action {:?}", other),
        }
    }

    #[test]
    fn create_plant_test_from_orbit_to_other_orbit() {
        let mut world = World::new();

        let sector_id = world.spawn_empty().id();

        let starting_asteroid_id = world
            .spawn_empty()
            .insert(LocationSpace {
                sector_id: sector_id,
                pos: P2::new(0.0, 0.0),
            })
            .id();

        let target_asteroid_pos = P2::new(1.0, 0.0);
        let target_asteroid_id = world
            .spawn_empty()
            .insert(LocationSpace {
                sector_id: sector_id,
                pos: target_asteroid_pos,
            })
            .id();

        let fleet_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::new(0.0, 0.0),
                sector_id: sector_id,
            })
            .insert(LocationOrbit {
                parent_id: starting_asteroid_id,
                distance: 0.0,
                start_time: Default::default(),
                start_angle: 0.0,
                speed: Speed(0.0),
            })
            .id();

        let plan = world
            .run_system_once_with(
                (
                    fleet_id,
                    NavRequest::OrbitTarget {
                        target_id: target_asteroid_id,
                    },
                ),
                create_plan_system,
            )
            .expect("fail to generate plan");

        assert_eq!(plan.path.len(), 3);
        match &plan.path[0] {
            Action::Deorbit => {}
            other => panic!("unexpected action {:?}", other),
        }
        match &plan.path[1] {
            Action::MoveToTargetPos {
                target_id: plan_target_id,
                ..
            } => {
                assert_eq!(target_asteroid_id, *plan_target_id);
            }
            other => panic!("found {:?}", other),
        }
        match &plan.path[2] {
            Action::Orbit {
                target_id: plan_target_id,
            } => {
                assert_eq!(target_asteroid_id, *plan_target_id);
            }
            other => panic!("unexpected action {:?}", other),
        }
    }
}
