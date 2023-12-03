use crate::game::actions::*;

use crate::game::navigations::navigation_request_handler_system::NavRequestHandlerSystem;
use crate::game::navigations::navigation_system::NavigationSystem;
use crate::game::objects::ObjId;
use crate::game::sectors::{Jump, Sector, SectorId};
use crate::game::{GameInitContext, RequireInitializer};

use bevy_ecs::prelude::*;

use crate::game::locations::{LocationDocked, LocationOrbit, LocationSpace, Locations};
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

// #[derive(Debug, Clone)]
// pub enum PlanTarget {
//     Pos(P2),
//     ObjPos(ObjId),
//     DockAt(ObjId),
//     Orbit(ObjId),
// }

#[derive(Debug, Clone)]
pub struct NavigationPlan {
    pub path: VecDeque<Action>,
}

// impl NavigationPlan {
//     pub fn append_dock(&mut self, target_id: ObjId) {
//         self.path.push_back(Action::Dock { target_id });
//     }
// }

// impl NavigationPlan {
//     pub fn take_next(&mut self) -> Option<Action> {
//         self.plan.path.pop_front()
//     }
// }

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

// pub fn resolve_target_sector_id_and_post<'a>(
//     locations: &readstorage<'a, locationspace>,
//     location_docked: &readstorage<'a, locationdocked>,
// target_id: objid) ->  result<locationspace, &'static str> {
//     locations::resolve_space_position(locations, location_docked, *target_id)
//         .ok_or("provided obj has no location")?;
// }

pub fn create_plan<'a>(
    entities: &Entities<'a>,
    sectors: &ReadStorage<'a, Sector>,
    jumps: &ReadStorage<'a, Jump>,
    locations: &ReadStorage<'a, LocationSpace>,
    location_docked: &ReadStorage<'a, LocationDocked>,
    location_orbiting: &ReadStorage<'a, LocationOrbit>,
    obj_id: Entity,
    request: &NavRequest,
) -> Result<NavigationPlan, &'static str> {
    let mut path = VecDeque::new();
    if location_docked.contains(obj_id) {
        path.push_back(Action::Undock);
    }
    if location_orbiting.contains(obj_id) {
        path.push_back(Action::Deorbit);
    }

    let from_location = Locations::resolve_space_position(locations, location_docked, obj_id)
        .ok_or("provided obj has no location")?;

    let to_location = match request {
        NavRequest::OrbitTarget { target_id } => {
            Locations::resolve_space_position(locations, location_docked, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveToTarget { target_id } => {
            Locations::resolve_space_position(locations, location_docked, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveAndDockAt { target_id } => {
            Locations::resolve_space_position(locations, location_docked, *target_id)
                .ok_or("provided obj has no location")?
        }
        NavRequest::MoveToPos { sector_id, pos } => LocationSpace {
            sector_id: *sector_id,
            pos: *pos,
        },
    };

    let sector_path = super::sectors::find_path(
        entities,
        sectors,
        jumps,
        locations,
        from_location.sector_id,
        to_location.sector_id,
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
    use crate::utils::Speed;

    #[test]
    fn create_plant_test_from_docked_into_other_dock() {
        let mut world = World::new();
        world.register::<LocationOrbit>();
        world.register::<LocationDocked>();

        let scn = setup_sector_scenery(&mut world);

        let starting_station_id = world
            .create_entity()
            .with(LocationSpace {
                sector_id: scn.sector_0,
                pos: P2::new(0.0, 0.0),
            })
            .build();

        let target_station_id = world
            .create_entity()
            .with(LocationSpace {
                sector_id: scn.sector_1,
                pos: P2::new(1.0, 0.0),
            })
            .build();

        let fleet_id = world
            .create_entity()
            .with(LocationDocked {
                parent_id: starting_station_id,
            })
            .build();

        let entities = world.entities();
        let sectors = world.read_storage::<Sector>();
        let jumps = world.read_storage::<Jump>();
        let locations = world.read_storage::<LocationSpace>();
        let locations_docked = world.read_storage::<LocationDocked>();
        let locations_orbit = world.read_storage::<LocationOrbit>();

        let plan = create_plan(
            &entities,
            &sectors,
            &jumps,
            &locations,
            &locations_docked,
            &locations_orbit,
            fleet_id,
            &NavRequest::MoveAndDockAt {
                target_id: target_station_id,
            },
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

        world.register::<LocationSpace>();
        world.register::<LocationOrbit>();
        world.register::<LocationDocked>();
        world.register::<Sector>();
        world.register::<Jump>();

        let sector_id = world.create_entity().build();

        let starting_asteroid_id = world
            .create_entity()
            .with(LocationSpace {
                sector_id: sector_id,
                pos: P2::new(0.0, 0.0),
            })
            .build();

        let target_asteroid_pos = P2::new(1.0, 0.0);
        let target_asteroid_id = world
            .create_entity()
            .with(LocationSpace {
                sector_id: sector_id,
                pos: target_asteroid_pos,
            })
            .build();

        let fleet_id = world
            .create_entity()
            .with(LocationSpace {
                pos: P2::new(0.0, 0.0),
                sector_id: sector_id,
            })
            .with(LocationOrbit {
                parent_id: starting_asteroid_id,
                distance: 0.0,
                start_time: Default::default(),
                start_angle: 0.0,
                speed: Speed(0.0),
            })
            .build();

        let entities = world.entities();
        let sectors = world.read_storage::<Sector>();
        let jumps = world.read_storage::<Jump>();
        let locations = world.read_storage::<LocationSpace>();
        let locations_docked = world.read_storage::<LocationDocked>();
        let locations_orbit = world.read_storage::<LocationOrbit>();

        let plan = create_plan(
            &entities,
            &sectors,
            &jumps,
            &locations,
            &locations_docked,
            &locations_orbit,
            fleet_id,
            &NavRequest::OrbitTarget {
                target_id: target_asteroid_id,
            },
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
