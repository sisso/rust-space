use specs::prelude::*;

use super::super::locations::*;
use super::*;
use std::borrow::{Borrow, BorrowMut};

///
/// Setup navigation for the request
/// - check for inconsistencies
///
pub struct NavRequestHandlerSystem;

#[derive(SystemData)]
pub struct NavRequestHandlerData<'a> {
    entities: Entities<'a>,
    sectors: Read<'a, SectorsIndex>,
    locations: ReadStorage<'a, Location>,
    requests: WriteStorage<'a, NavRequest>,
    navigation: WriteStorage<'a, Navigation>,
    navigation_move_to: WriteStorage<'a, NavigationMoveTo>,
}

impl<'a> System<'a> for NavRequestHandlerSystem {
    type SystemData = NavRequestHandlerData<'a>;

    fn run(&mut self, mut data: NavRequestHandlerData) {
        trace!("running");

        let sectors = data.sectors.borrow();

        let mut processed_requests = vec![];
        let locations = data.locations.borrow();

        for (entity, request, location) in (&*data.entities, &data.requests, &data.locations).join()
        {
            let (target_id, should_dock) = match request {
                NavRequest::MoveToTarget { target_id, } => (*target_id, false),
                NavRequest::MoveAndDockAt { target_id, } => (*target_id, true),
            };

            processed_requests.push(entity);

            let is_docked = location.as_docked().is_some();
            let location = Locations::resolve_space_position(locations, entity)
                .expect("entity has no location");
            let target_location = Locations::resolve_space_position(locations, target_id)
                .expect("target has no location");

            let mut plan = Navigations::create_plan(
                sectors,
                location.sector_id,
                location.pos,
                target_location.sector_id,
                target_location.pos,
                is_docked,
            );

            if should_dock {
                plan.append_dock(target_id);
            }

            debug!("{:?} handle navigation to {:?} by the plan {:?}", entity, request, plan);

            data.navigation.insert(entity, Navigation::MoveTo).unwrap();
            data.navigation_move_to
                .insert(entity, NavigationMoveTo { target_id, plan, })
                .unwrap();
        }

        let request_storage = &mut data.requests;
        for e in processed_requests {
            request_storage.remove(e).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;
    use crate::test::test_system;

    fn setup_station_and_asteroid(world: &mut World, sectors: &SectorScenery) -> (Entity, Entity) {
        let asteroid = world
            .create_entity()
            .with(Location::Space {
                pos: Position::new(1.0, 0.0),
                sector_id: sectors.sector_1,
            })
            .build();

        let station = world
            .create_entity()
            .with(Location::Space {
                pos: Position::new(0.0, 0.0),
                sector_id: sectors.sector_0,
            })
            .build();

        (station, asteroid)
    }

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests() {
        let (world, (asteroid, miner)) = test_system(NavRequestHandlerSystem, |world| {
            let sector_scenery = crate::game::sectors::test_scenery::setup_sector_scenery(world);
            let (station, asteroid) = setup_station_and_asteroid(world, &sector_scenery);

            let miner = world
                .create_entity()
                .with(Location::Space {
                    pos: Position::new(0.0, 0.0),
                    sector_id: sector_scenery.sector_0,
                })
                .with(NavRequest::MoveToTarget {
                    target_id: asteroid,
                })
                .build();

            (asteroid, miner)
        });

        let nav_storage = world.read_component::<Navigation>();
        let nav = nav_storage.get(miner).unwrap();
        assert_eq!(nav.clone(), Navigation::MoveTo);

        let nav_move_to_storage = world.read_component::<NavigationMoveTo>();
        let nav_move_to = nav_move_to_storage.get(miner).unwrap();
        assert_eq!(nav_move_to.target_id, asteroid);

        let nav_request_storage = world.read_component::<NavRequest>();
        let nav_request = nav_request_storage.get(miner);
        assert!(nav_request.is_none());
    }

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests_when_docked() {
        let (world, (asteroid, miner)) = test_system(NavRequestHandlerSystem, |world| {
            let sector_scenery = crate::game::sectors::test_scenery::setup_sector_scenery(world);
            let (station, asteroid) = setup_station_and_asteroid(world, &sector_scenery);

            let miner = world
                .create_entity()
                .with(Location::Dock { docked_id: station })
                .with(NavRequest::MoveToTarget {
                    target_id: asteroid,
                })
                .build();

            (asteroid, miner)
        });

        let nav_storage = world.read_component::<Navigation>();
        let nav = nav_storage.get(miner).unwrap();
        assert_eq!(nav.clone(), Navigation::MoveTo);
    }
}
