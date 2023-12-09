use bevy_ecs::prelude::*;

use super::*;
use crate::game::events::{Event, EventKind, Events};
use crate::game::locations::LocationSpace;
use crate::game::sectors::Jump;
use std::borrow::BorrowMut;

pub struct ActionJumpSystem;

#[derive(SystemData)]
pub struct ActionJumpData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    // sectors: Read<'a, Sectors>,
    actions: WriteStorage<'a, ActionActive>,
    actions_jump: WriteStorage<'a, ActionJump>,
    locations_space: WriteStorage<'a, LocationSpace>,
    jumps: ReadStorage<'a, Jump>,
    events: Write<'a, Events>,
}

impl<'a> System<'a> for ActionJumpSystem {
    type SystemData = ActionJumpData<'a>;

    fn run(&mut self, mut data: ActionJumpData) {
        log::trace!("running");

        let total_time: TotalTime = *data.total_time;

        let mut completed = vec![];

        for (entity, action, action_jump) in
            (&*data.entities, &data.actions, &mut data.actions_jump).join()
        {
            let jump_id = match action.get_action() {
                Action::Jump { jump_id } => jump_id.clone(),
                other => {
                    log::warn!(
                        "{:?} has jump action component but action {:?} is not jump",
                        entity,
                        other,
                    );
                    continue;
                }
            };

            match action_jump.complete_time {
                Some(value) if value.is_before(total_time) => {
                    completed.push((entity, jump_id));
                }
                Some(_) => {
                    log::trace!("{:?} jumping", entity);
                }
                None => {
                    log::debug!("{:?} start to jump", entity);
                    action_jump.complete_time = Some(total_time.add(ACTION_JUMP_TOTAL_TIME));
                }
            }
        }

        let actions = data.actions.borrow_mut();
        let actions_jump = data.actions_jump.borrow_mut();
        let events = data.events.borrow_mut();

        for (entity, jump_id) in completed {
            let jump = data.jumps.get(jump_id).unwrap();

            log::debug!(
                "{:?} jump complete to sector {:?} at position {:?}",
                entity,
                jump.target_sector_id,
                jump.target_pos,
            );

            data.locations_space
                .borrow_mut()
                .insert(
                    entity,
                    LocationSpace {
                        pos: jump.target_pos,
                        sector_id: jump.target_sector_id,
                    },
                )
                .unwrap();

            actions.remove(entity).unwrap();
            actions_jump.remove(entity).unwrap();
            events.push(Event::new(entity, EventKind::Jump));
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::game::sectors::test_scenery;
    use crate::game::sectors::test_scenery::SectorScenery;
    use crate::game::utils::TotalTime;
    use crate::test::{assert_v2, test_system};

    fn create_jump_entity(
        world: &mut World,
        scenery: &SectorScenery,
        jump_time: Option<TotalTime>,
    ) -> Entity {
        let entity = world
            .create_entity()
            .insert(ActionActive(Action::Jump {
                jump_id: scenery.jump_0_to_1,
            }))
            .insert(ActionJump {
                complete_time: jump_time,
            })
            .insert(LocationSpace {
                pos: scenery.jump_0_to_1_pos,
                sector_id: scenery.sector_0,
            })
            .id();
        entity
    }

    fn assert_jumped(world: &World, sector_scenery: &SectorScenery, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionJump>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_1);
        assert_v2(location.pos, sector_scenery.jump_1_to_0_pos);
    }

    fn assert_not_jumped(world: &World, sector_scenery: &SectorScenery, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionJump>().get(entity).is_some());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_0);
        assert_v2(location.pos, sector_scenery.jump_0_to_1_pos);
    }

    #[test]
    fn test_jump_system_should_set_total_time_if_not_defined() {
        let initial_time = TotalTime(1.0);

        let (world, (entity, sector_scenery)) = test_system(ActionJumpSystem, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert(initial_time);
            (
                create_jump_entity(world, &sectors_scenery, None),
                sectors_scenery,
            )
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
            (
                create_jump_entity(world, &sectors_scenery, Some(TotalTime(1.5))),
                sectors_scenery,
            )
        });

        assert_not_jumped(&world, &sector_scenery, entity);
    }

    #[test]
    fn test_jump_system_should_jump() {
        let (world, (entity, sector_scenery)) = test_system(ActionJumpSystem, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert(TotalTime(1.0));
            (
                create_jump_entity(world, &sectors_scenery, Some(TotalTime(0.5))),
                sectors_scenery,
            )
        });

        assert_jumped(&world, &sector_scenery, entity);
    }
}
