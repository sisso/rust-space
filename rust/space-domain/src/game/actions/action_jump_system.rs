use bevy_ecs::prelude::*;

use super::*;
use crate::game::events::{CommandSendEvent, EventKind, GEvent};
use crate::game::locations::{LocationDocked, LocationSpace};
use crate::game::sectors::Jump;

pub struct ActionJumpSystem;

pub fn system_jump(
    mut commands: Commands,
    total_time: Res<TotalTime>,
    mut query: Query<(Entity, &ActionActive, &mut ActionJump)>,
    _query_locations: Query<(Entity, Option<&LocationSpace>, Option<&LocationDocked>)>,
    query_jumps: Query<&Jump>,
) {
    log::trace!("running");

    let total_time = *total_time;

    for (obj_id, action, mut action_jump) in &mut query {
        let jump_id = match action.get_action() {
            Action::Jump { jump_id } => jump_id.clone(),
            other => {
                log::warn!(
                    "{:?} has jump action component but action {:?} is not jump",
                    obj_id,
                    other,
                );
                continue;
            }
        };

        match action_jump.complete_time {
            Some(value) if value.is_before(total_time) => {
                let jump = query_jumps.get(jump_id).unwrap();

                log::debug!(
                    "{:?} jump complete to sector {:?} at position {:?}",
                    obj_id,
                    jump.target_sector_id,
                    jump.target_pos,
                );

                commands
                    .entity(obj_id)
                    .insert(LocationSpace {
                        pos: jump.target_pos,
                        sector_id: jump.target_sector_id,
                    })
                    .remove::<ActionActive>()
                    .remove::<ActionJump>();

                commands.add(CommandSendEvent::from(GEvent::new(obj_id, EventKind::Jump)));
            }
            Some(_) => {
                log::trace!("{:?} jumping", obj_id);
            }
            None => {
                log::debug!("{:?} start to jump", obj_id);
                action_jump.complete_time = Some(total_time.add(ACTION_JUMP_TOTAL_TIME));
            }
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
            .spawn_empty()
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
        assert!(world.get::<ActionActive>(entity).is_none());
        assert!(world.get::<ActionJump>(entity).is_none());
        let location = world.get::<LocationSpace>(entity).unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_1);
        assert_v2(location.pos, sector_scenery.jump_1_to_0_pos);
    }

    fn assert_not_jumped(world: &World, sector_scenery: &SectorScenery, entity: Entity) {
        assert!(world.get::<ActionActive>(entity).is_some());
        assert!(world.get::<ActionJump>(entity).is_some());
        let location = world.get::<LocationSpace>(entity).unwrap();
        assert_eq!(location.sector_id, sector_scenery.sector_0);
        assert_v2(location.pos, sector_scenery.jump_0_to_1_pos);
    }

    #[test]
    fn test_jump_system_should_set_total_time_if_not_defined() {
        let initial_time = TotalTime(1.0);

        let (world, (entity, sector_scenery)) = test_system(system_jump, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert_resource(initial_time);
            (
                create_jump_entity(world, &sectors_scenery, None),
                sectors_scenery,
            )
        });

        assert_not_jumped(&world, &sector_scenery, entity);

        let action = world.get::<ActionJump>(entity).unwrap();
        assert_eq!(
            action.complete_time.unwrap().as_f64(),
            initial_time.add(ACTION_JUMP_TOTAL_TIME).as_f64()
        );
    }

    #[test]
    fn test_jump_system_should_take_time() {
        let (world, (entity, sector_scenery)) = test_system(system_jump, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert_resource(TotalTime(1.0));
            (
                create_jump_entity(world, &sectors_scenery, Some(TotalTime(1.5))),
                sectors_scenery,
            )
        });

        assert_not_jumped(&world, &sector_scenery, entity);
    }

    #[test]
    fn test_jump_system_should_jump() {
        let (world, (entity, sector_scenery)) = test_system(system_jump, |world| {
            let sectors_scenery = test_scenery::setup_sector_scenery(world);
            world.insert_resource(TotalTime(1.0));
            (
                create_jump_entity(world, &sectors_scenery, Some(TotalTime(0.5))),
                sectors_scenery,
            )
        });

        assert_jumped(&world, &sector_scenery, entity);
    }
}
