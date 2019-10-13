use specs::prelude::*;

use super::*;
use super::super::locations::*;
use std::borrow::{Borrow, BorrowMut};

///
/// Setup navigation for the request
/// - check for inconsistencies
///
pub struct NavRequestHandlerSystem;

#[derive(SystemData)]
pub struct NavRequestHandlerData<'a> {
    entities: Entities<'a>,
    sectors: Read<'a, Sectors>,
    locations_sector_id: ReadStorage<'a, LocationSector>,
    locations_positions: ReadStorage<'a, LocationSpace>,
    requests: WriteStorage<'a , NavRequest>,
    navigation: WriteStorage<'a , Navigation>,
    navigation_move_to: WriteStorage<'a , NavigationMoveTo>,
}

impl<'a> System<'a> for NavRequestHandlerSystem {
    type SystemData = NavRequestHandlerData<'a>;

    fn run(&mut self, mut data: NavRequestHandlerData) {
        let sectors = data.sectors.borrow();

        let mut processed_requests = vec![];

        for (entity, request, from_sector_id, from_pos) in (&data.entities, &data.requests, &data.locations_sector_id, &data.locations_positions).join() {
            match request {
                NavRequest::MoveToTarget { target } => {
                    let target_sector_id = data.locations_sector_id.borrow().get(*target).unwrap();
                    let target_pos = data.locations_positions.borrow().get(*target).unwrap();

                    let plan = Navigations::create_plan(
                        sectors,
                        from_sector_id.sector_id,
                        from_pos.pos.clone(),
                        target_sector_id.sector_id,
                        target_pos.pos
                    );

                    let _ = data.navigation.insert(entity, Navigation::MoveTo).unwrap();
                    let _ = data.navigation_move_to.insert(entity, NavigationMoveTo {
                        target: *target,
                        plan,
                    }).unwrap();

                    processed_requests.push(entity);
                },
                _ => panic!("unsupported"),
            }
        }

        let request_storage = data.requests.borrow_mut();
        for e in processed_requests {
            request_storage.remove(e).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::sectors::test_scenery::*;

    #[test]
    fn test_nav_request_handler_should_create_navigation_from_requests() {
        let mut world = World::new();
        world.insert(new_test_sectors());

        let mut dispatcher = DispatcherBuilder::new()
            .with(NavRequestHandlerSystem, "nav_request_handler_system", &[])
            .build();
        dispatcher.setup(&mut world);

        let asteroid = world.create_entity()
            .with(LocationSector { sector_id: SECTOR_1 })
            .with(LocationSpace { pos: Position::new(1.0, 0.0) })
            .build();

        let miner = world.create_entity()
            .with(LocationSector { sector_id: SECTOR_0 })
            .with(LocationSpace { pos: Position::new(0.0, 0.0) })
            .with(NavRequest::MoveToTarget { target: asteroid })
            .build();

        dispatcher.dispatch(&world);
        world.maintain();

        let nav_storage = world.read_component::<Navigation>();
        let nav = nav_storage.get(miner).unwrap();
        assert_eq!(nav.clone(), Navigation::MoveTo);

        let nav_move_to_storage = world.read_component::<NavigationMoveTo>();
        let nav_move_to = nav_move_to_storage.get(miner).unwrap();
        assert_eq!(nav_move_to.target, asteroid);

        let nav_request_storage = world.read_component::<NavRequest>();
        let nav_request = nav_request_storage.get(miner);
        assert!(nav_request.is_none());
    }
}
