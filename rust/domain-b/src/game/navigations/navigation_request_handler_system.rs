use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;
use crate::game::sectors::{Jump, Sector};
use crate::game::SYSTEM_TIMEOUT;

///
/// Setup navigation for the request
/// - check for inconsistencies
///
pub struct NavRequestHandlerSystem;

#[derive(SystemData)]
pub struct NavRequestHandlerData<'a> {
    entities: Entities<'a>,
    sectors: ReadStorage<'a, Sector>,
    jumps: ReadStorage<'a, Jump>,
    locations: ReadStorage<'a, LocationSpace>,
    locations_docked: ReadStorage<'a, LocationDocked>,
    locations_orbit: ReadStorage<'a, LocationOrbit>,
    requests: WriteStorage<'a, NavRequest>,
    navigation: WriteStorage<'a, Navigation>,
}

impl<'a> System<'a> for NavRequestHandlerSystem {
    fn run(&mut self, mut data: NavRequestHandlerData) {
        log::trace!("running");

        let mut processed_requests = vec![];

        // timeout navigation handling if take more time that expected
        let timeout = commons::TimeDeadline::new(SYSTEM_TIMEOUT);

        for (id, request) in (&*data.entities, &data.requests).join() {
            processed_requests.push(id);

            let plan = match create_plan(
                &data.entities,
                &data.sectors,
                &data.jumps,
                &data.locations,
                &data.locations_docked,
                &data.locations_orbit,
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

            data.navigation
                .insert(
                    id,
                    Navigation {
                        request: request.clone(),
                        plan,
                    },
                )
                .unwrap();

            if timeout.is_timeout() {
                log::warn!("navigation request timeout");
                break;
            }
        }

        let request_storage = &mut data.requests;
        for e in processed_requests {
            request_storage.remove(e).unwrap();
        }
    }

    type SystemData = NavRequestHandlerData<'a>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;
    use crate::test::test_system;

    fn setup_station_and_asteroid(world: &mut World, sectors: &SectorScenery) -> (Entity, Entity) {
        let asteroid = world
            .create_entity()
            .insert(LocationSpace {
                pos: P2::X,
                sector_id: sectors.sector_1,
            })
            .id();

        let station = world
            .create_entity()
            .insert(LocationSpace {
                pos: P2::ZERO,
                sector_id: sectors.sector_0,
            })
            .id();

        (station, asteroid)
    }

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests() {
        let (world, (asteroid_id, miner)) = test_system(NavRequestHandlerSystem, |world| {
            let sector_scenery = crate::game::sectors::test_scenery::setup_sector_scenery(world);
            let (_station, asteroid) = setup_station_and_asteroid(world, &sector_scenery);

            let miner = world
                .create_entity()
                .insert(LocationSpace {
                    pos: P2::ZERO,
                    sector_id: sector_scenery.sector_0,
                })
                .insert(NavRequest::MoveToTarget {
                    target_id: asteroid,
                })
                .id();

            (asteroid, miner)
        });

        let nav_storage = world.read_component::<Navigation>();
        let nav = nav_storage.get(miner).unwrap();
        assert_eq!(
            nav.request,
            NavRequest::MoveToTarget {
                target_id: asteroid_id
            }
        );

        let nav_request_storage = world.read_component::<NavRequest>();
        let nav_request = nav_request_storage.get(miner);
        assert!(nav_request.is_none());
    }

    // #[test]
    // fn test_nav_request_handler_should_create_navigation_from_requests_when_docked() {
    //     let (world, (asteroid_id, miner_id)) = test_system(NavRequestHandlerSystem, |world| {
    //         let sector_scenery = setup_sector_scenery(world);
    //         let (station, asteroid) = setup_station_and_asteroid(world, &sector_scenery);
    //
    //         let miner = world
    //             .create_entity()
    //             .insert(LocationDocked { parent_id: station })
    //             .insert(NavRequest::MoveToTarget {
    //                 target_id: asteroid,
    //             })
    //             .id();
    //
    //         (asteroid, miner)
    //     });
    //
    //     let nav_storage = world.read_component::<Navigation>();
    //     let nav = nav_storage.get(miner_id).unwrap();
    //     assert_eq!(
    //         nav.request,
    //         NavRequest::MoveToTarget {
    //             target_id: asteroid_id,
    //         }
    //     );
    // }
    //
    // #[test]
    // fn test_nav_request_handler_should_remove_orbiting_before_move() {
    //     let (world, (asteroid_id, miner_id)) = test_system(NavRequestHandlerSystem, |world| {
    //         let sector_scenery = setup_sector_scenery(world);
    //         let (station_id, asteroid_id) = setup_station_and_asteroid(world, &sector_scenery);
    //
    //         let miner = world
    //             .create_entity()
    //             .insert(LocationSpace {
    //                 sector_id: sector_scenery.sector_0,
    //                 pos: P2::X,
    //             })
    //             .insert(LocationOrbit {
    //                 parent_id: asteroid_id,
    //                 distance: 0.0,
    //                 start_time: Default::default(),
    //                 start_angle: 0.0,
    //                 speed: Speed(0.0),
    //             })
    //             .insert(NavRequest::MoveToTarget {
    //                 target_id: station_id,
    //             })
    //             .id();
    //
    //         (asteroid_id, miner)
    //     });
    //
    //     assert!(world
    //         .read_component::<LocationOrbit>()
    //         .get(miner_id)
    //         .is_none());
    //
    //     assert_eq!(
    //         world
    //             .read_component::<Navigation>()
    //             .get(miner_id)
    //             .unwrap()
    //             .request
    //             .clone(),
    //         NavRequest::MoveToTarget {
    //             target_id: asteroid_id,
    //         }
    //     );
    // }
    //
    // #[test]
    // fn test_nav_request_handler_should_navigate_to_orbit() {
    //     let (world, (asteroid_id, miner_id)) = test_system(NavRequestHandlerSystem, |world| {
    //         let sector_scenery = setup_sector_scenery(world);
    //         let asteroid_id = world
    //             .create_entity()
    //             .insert(LocationSpace {
    //                 pos: P2::X,
    //                 sector_id: sector_scenery.sector_1,
    //             })
    //             .id();
    //
    //         let miner = world
    //             .create_entity()
    //             .insert(LocationSpace {
    //                 sector_id: sector_scenery.sector_0,
    //                 pos: P2::ZERO,
    //             })
    //             .insert(NavRequest::OrbitTarget {
    //                 target_id: asteroid_id,
    //             })
    //             .id();
    //
    //         (asteroid_id, miner)
    //     });
    //
    //     assert!(world
    //         .read_component::<LocationOrbit>()
    //         .get(miner_id)
    //         .is_none());
    //
    //     assert_eq!(
    //         world
    //             .read_component::<Navigation>()
    //             .get(miner_id)
    //             .map(|i| i.request.clone()),
    //         Some(NavRequest::OrbitTarget {
    //             target_id: asteroid_id,
    //         })
    //     );
    // }
}
