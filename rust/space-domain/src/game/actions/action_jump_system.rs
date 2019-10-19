use specs::prelude::*;

use crate::game::locations::{LocationSector, LocationSpace};
use super::*;
use std::borrow::{Borrow, BorrowMut};
use crate::game::sectors::Sectors;

pub struct ActionJumpSystem;

#[derive(SystemData)]
pub struct ActionJumpData<'a> {
    entities: Entities<'a>,
    total_time: Read<'a, TotalTime>,
    sectors: Read<'a, Sectors>,
    actions: WriteStorage<'a, ActionActive>,
    actions_jump: WriteStorage<'a, ActionJump>,
    locations_sector: WriteStorage<'a, LocationSector>,
    locations_space: WriteStorage<'a, LocationSpace>,
}

impl<'a> System<'a> for ActionJumpSystem {
    type SystemData = ActionJumpData<'a>;

    fn run(&mut self, mut data: ActionJumpData) {
        let total_time: TotalTime = *data.total_time;

        let mut completed = vec![];

        for (entity, action, action_jump)  in (&*data.entities, &data.actions, &mut data.actions_jump).join() {
            let to_jump_id = match action.get_action() {
                Action::Jump { jump_id } => jump_id.clone(),
                _ => continue,
            };

            match action_jump.complete_time {
                Some(value) if value.is_before(total_time) => {
                    completed.push((entity, to_jump_id));
                },
                Some(_) => {},
                None => {
                    action_jump.complete_time = Some(total_time.add(ACTION_JUMP_TOTAL_TIME));
                }
            }
        }

        let sectors = data.sectors.borrow();

        for (entity, jump_id) in completed {
            let jump = sectors.get_jump(jump_id).unwrap();
            let _ = data.locations_space.borrow_mut().insert(entity, LocationSpace { pos: jump.to_pos });
            let _ = data.locations_sector.borrow_mut().insert(entity, LocationSector { sector_id: jump.to_sector_id });
            let _ = data.actions.borrow_mut().remove(entity);
            let _ = data.actions_jump.borrow_mut().remove(entity);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::test::{test_system, assert_v2};
    use crate::utils::{Speed, TotalTime};
    use crate::game::locations::LocationSpace;
    use crate::game::sectors::test_scenery;

    fn create_jump_entity(world: &mut World, jump_time: Option<TotalTime>) -> Entity {
        let entity = world.create_entity()
            .with(ActionActive(Action::Jump { jump_id: test_scenery::JUMP_0_TO_1.id }))
            .with(ActionJump { complete_time: jump_time })
            .with(LocationSpace { pos: test_scenery::JUMP_0_TO_1.pos })
            .with(LocationSector { sector_id: test_scenery::SECTOR_0 })
            .build();
        entity
    }

    fn assert_jumped(world: &World, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionJump>().get(entity).is_none());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, test_scenery::JUMP_0_TO_1.to_pos);
        let storage = world.read_storage::<LocationSector>();
        let location = storage.get(entity).unwrap();
        assert_eq!(location.sector_id, test_scenery::JUMP_0_TO_1.to_sector_id);
    }

    fn assert_not_jumped(world: &World, entity: Entity) {
        assert!(world.read_storage::<ActionActive>().get(entity).is_some());
        assert!(world.read_storage::<ActionJump>().get(entity).is_some());
        let storage = world.read_storage::<LocationSpace>();
        let location = storage.get(entity).unwrap();
        assert_v2(location.pos, test_scenery::JUMP_0_TO_1.pos);
        let storage = world.read_storage::<LocationSector>();
        let location = storage.get(entity).unwrap();
        assert_eq!(location.sector_id, test_scenery::JUMP_0_TO_1.sector_id);
    }

    #[test]
    fn test_jump_system_should_set_total_time_if_not_defined() {
        let initial_time = TotalTime(1.0);

        let (world, entity) = test_system(ActionJumpSystem, |world| {
            world.insert(test_scenery::new_test_sectors());
            world.insert(initial_time);
            create_jump_entity(world, None)
        });

        assert_not_jumped(&world, entity);

        let storage = world.read_storage::<ActionJump>();
        let action= storage.get(entity).unwrap();
        assert_eq!(action.complete_time.unwrap().as_f64(), initial_time.add(ACTION_JUMP_TOTAL_TIME).as_f64());
    }

    #[test]
    fn test_jump_system_should_take_time() {
        let (world, entity) = test_system(ActionJumpSystem, |world| {
            world.insert(test_scenery::new_test_sectors());
            world.insert(TotalTime(1.0));
            create_jump_entity(world, Some(TotalTime(1.1)))
        });

        assert_not_jumped(&world, entity);
    }

    #[test]
    fn test_jump_system_should_jump() {
        let (world, entity) = test_system(ActionJumpSystem, |world| {
            world.insert(test_scenery::new_test_sectors());
            world.insert(TotalTime(1.0));
            create_jump_entity(world, Some(TotalTime(0.5)))
        });

        assert_jumped(&world, entity);
    }
}
