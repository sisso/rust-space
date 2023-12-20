use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;
use crate::game::game::SYSTEM_TIMEOUT;
use crate::game::sectors::{Jump, Sector};

///
/// Setup navigation for the request
/// - check for inconsistencies
///
pub fn system_navigation_request(
    mut commands: Commands,
    query: Query<(Entity, &NavRequest)>,
    query_entity: Query<(Option<&LocationDocked>, Option<&LocationOrbit>)>,
    query_locations: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    query_sectors: Query<&Sector>,
    query_jumps: Query<(&Jump, &LocationSpace)>,
) {
    log::trace!("running");

    let mut processed_requests = vec![];

    // timeout navigation handling if take more time that expected
    let timeout = commons::TimeDeadline::new(SYSTEM_TIMEOUT);

    for (id, request) in &query {
        processed_requests.push(id);

        let plan = match create_plan(
            &query_entity,
            &query_locations,
            &query_sectors,
            &query_jumps,
            id,
            request,
        ) {
            Ok(plan) => plan,
            Err(err) => {
                log::warn!(
                    "{:?} fail to generate navigation plan for {:?}: {}",
                    id,
                    request,
                    err
                );
                continue;
            }
        };

        log::debug!(
            "{:?} handle navigation to {:?} by the plan {:?}",
            id,
            request,
            plan,
        );

        commands.entity(id).insert(Navigation {
            request: request.clone(),
            plan,
        });

        if timeout.is_timeout() {
            log::warn!("navigation request timeout");
            break;
        }
    }

    for obj_id in processed_requests {
        commands
            .get_entity(obj_id)
            .expect("processed obj not found")
            .remove::<NavRequest>();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_station_and_asteroid(world: &mut World, sectors: &SectorScenery) -> (Entity, Entity) {
        let asteroid = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::X,
                sector_id: sectors.sector_1,
            })
            .id();

        let station = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::ZERO,
                sector_id: sectors.sector_0,
            })
            .id();

        (station, asteroid)
    }

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests() {
        let mut world = World::new();

        let sector_scenery = setup_sector_scenery(&mut world);
        let (_station_id, asteroid_id) = setup_station_and_asteroid(&mut world, &sector_scenery);

        let miner_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: P2::ZERO,
                sector_id: sector_scenery.sector_0,
            })
            .insert(NavRequest::MoveToTarget {
                target_id: asteroid_id,
            })
            .id();

        world.run_system_once(system_navigation_request);

        let nav = world.get::<Navigation>(miner_id).unwrap();
        assert_eq!(
            nav.request,
            NavRequest::MoveToTarget {
                target_id: asteroid_id
            }
        );

        assert!(world.get::<NavRequest>(miner_id).is_none());
    }

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests_when_docked() {
        let mut world = World::new();
        let sector_scenery = setup_sector_scenery(&mut world);
        let (station_id, asteroid_id) = setup_station_and_asteroid(&mut world, &sector_scenery);

        let miner_id = world
            .spawn_empty()
            .insert(LocationDocked {
                parent_id: station_id,
            })
            .insert(NavRequest::MoveToTarget {
                target_id: asteroid_id,
            })
            .id();

        world.run_system_once(system_navigation_request);

        assert_eq!(
            world.get::<Navigation>(miner_id).unwrap().request,
            NavRequest::MoveToTarget {
                target_id: asteroid_id,
            }
        );
    }
}
