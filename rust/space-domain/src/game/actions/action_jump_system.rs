use specs::prelude::*;

use super::*;
use crate::game::locations::Location;
use crate::game::sectors::{SectorsIndex, Jump};
use std::borrow::{Borrow, BorrowMut};
use crate::game::events::{Events, Event, EventKind};

pub struct ActionJumpSystem;

#[derive(SystemData)]
pub struct ActionJumpData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    // sectors: Read<'a, Sectors>,
    actions: WriteStorage<'a, ActionActive>,
    actions_jump: WriteStorage<'a, ActionJump>,
    locations: WriteStorage<'a, Location>,
    jumps: ReadStorage<'a, Jump>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for ActionJumpSystem {
    type SystemData = ActionJumpData<'a>;

    fn run(&mut self, mut data: ActionJumpData) {
        trace!("running");

        let total_time: TotalTime = *data.total_time;

        let mut completed = vec![];

        for (entity, action, action_jump) in
            (&*data.entities, &data.actions, &mut data.actions_jump).join()
        {
            let jump_id = match action.get_action() {
                Action::Jump { jump_id } => jump_id.clone(),
                other => {
                    warn!("{:?} has jump action component but action {:?} is not jump", entity, other);
                    continue
                },
            };

            match action_jump.complete_time {
                Some(value) if value.is_before(total_time) => {
                    completed.push((entity, jump_id));
                }
                Some(_) => {
                    trace!("{:?} jumping", entity);
                }
                None => {
                    debug!("{:?} start to jump", entity);
                    action_jump.complete_time = Some(total_time.add(ACTION_JUMP_TOTAL_TIME));
                }
            }
        }

        for (entity, jump_id) in completed {
            let current_jump = data.jumps.get(jump_id).unwrap();
            let target_jump_id = current_jump.target_id;
            let jump_location = data.locations.get(target_jump_id);

            let (to_pos, to_sector_id) = match jump_location {
                Some(Location::Space { pos, sector_id }) => (*pos, *sector_id),
                _ => panic!("{:?} target jump has a invalid location {:?}", target_jump_id, jump_location),
            };

            debug!(
                "{:?} jump complete to sector {:?} at position {:?}",
                entity, to_sector_id, to_pos
            );

            data.locations
                .borrow_mut()
                .insert(
                    entity,
                    Location::Space {
                        pos: to_pos,
                        sector_id: to_sector_id,
                    },
                )
                .unwrap();

            data.actions.borrow_mut().remove(entity).unwrap();
            data.actions_jump.borrow_mut().remove(entity).unwrap();
            data.events.borrow_mut().push(Event::new(entity, EventKind::Jump));
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::game::locations::Location;
    use crate::game::sectors::test_scenery;
    use crate::test::{assert_v2, test_system};
    use crate::utils::{Speed, TotalTime};
    use crate::game::sectors::test_scenery::SectorScenery;

    fn create_jump_entity(world: &mut World, scenery: &SectorScenery, jump_time: Option<TotalTime>) -> Entity {
        let entity = world
            .create_entity()
            .with(ActionActive(Action::Jump {
                jump_id: scenery.jump_0_to_1,
            }))
            .with(ActionJump {
                complete_time: jump_time,
            })
            .with(Location::Space {
                pos: scenery.jump_0_to_1_pos,
                sector_id: scenery.sector_0,
            })
            .build();
        entity
    }

    fn assert_jumped(world: &World, sector_scenery: &SectorScenery, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionJump>().get(entity).is_none());
        let storage = world.read_storage::<Location>();
        let location = storage.get(entity).unwrap().as_space().unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_1);
        assert_v2(location.pos, sector_scenery.jump_1_to_0_pos);
    }

    fn assert_not_jumped(world: &World, sector_scenery: &SectorScenery, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionJump>().get(entity).is_some());
        let storage = world.read_storage::<Location>();
        let location = storage.get(entity).unwrap().as_space().unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_0);
        assert_v2(location.pos, sector_scenery.jump_0_to_1_pos);
    }

    #[test]
    fn test_jump_system_should_set_total_time_if_not_defined() {
        let initial_time = TotalTime(1.0);

        let (world, (entity, sector_scenery)) = test_system(ActionJumpSystem, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert(initial_time);
            (create_jump_entity(world, &sectors_scenery, None), sectors_scenery)
        });

        assert_not_jumped(&world, &sector_scenery, entity);

        let storage = world.read_storage::<ActionJump>();
        let action = storage.get(entity).unwrap();
        assert_eq!(
            action.complete_time.unwrap().as_f64(),
            initial_time.add(ACTION_JUMP_TOTAL_TIME).as_f64()
        );
    }

    #[test]
    fn test_jump_system_should_take_time() {
        let (world, (entity, sector_scenery)) = test_system(ActionJumpSystem, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert(TotalTime(1.0));
            (create_jump_entity(world, &sectors_scenery, Some(TotalTime(1.5))), sectors_scenery)
        });

        assert_not_jumped(&world, &sector_scenery,  entity);
    }

    #[test]
    fn test_jump_system_should_jump() {
        let (world, (entity, sector_scenery)) = test_system(ActionJumpSystem, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert(TotalTime(1.0));
            (create_jump_entity(world, &sectors_scenery, Some(TotalTime(0.5))), sectors_scenery)
        });

        assert_jumped(&world, &sector_scenery, entity);
    }
}
