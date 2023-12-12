use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{CommandSendEvent, EventKind, GEvent};

use crate::game::dock::HasDocking;

pub fn system_undock(
    mut commands: Commands,
    query: Query<(Entity, Option<&LocationDocked>), With<ActionUndock>>,
    query_location: Query<&LocationSpace>,
    mut query_hasdock: Query<&mut HasDocking>,
) {
    log::trace!("running");

    for (obj_id, maybe_docked) in &query {
        if let Some(docked_at) = maybe_docked {
            match query_location.get(docked_at.parent_id).ok() {
                Some(location) => {
                    log::debug!("{:?} un-docking from {:?}", obj_id, docked_at.parent_id);
                    commands
                        .get_entity(obj_id)
                        .unwrap()
                        .insert(location.clone());
                    commands.add(CommandSendEvent::from(GEvent::new(
                        obj_id,
                        EventKind::Undock,
                    )));

                    query_hasdock
                        .get_mut(docked_at.parent_id)
                        .unwrap()
                        .docked
                        .retain(|i_id| *i_id != obj_id);
                }
                None => {
                    log::warn!("{:?} can not un-dock, parent is not in space", obj_id,);
                }
            }
        } else {
            log::warn!("{:?} can no undock, it is not docked, ignoring", obj_id);
        }

        // remove components
        commands
            .get_entity(obj_id)
            .unwrap()
            .remove::<LocationDocked>()
            .remove::<ActionActive>()
            .remove::<ActionUndock>();
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;

    use crate::game::utils::Position;
    use crate::test::{assert_v2, test_system};

    #[test]
    fn test_undock_system_should_undock_if_docked() {
        let (world, (fleet_id, station_id)) = test_system(system_undock, |world| {
            let sector_id = world.spawn_empty().id();

            let station_id = world
                .spawn_empty()
                .insert(LocationSpace {
                    pos: Position::new(0.0, 0.0),
                    sector_id: sector_id,
                })
                .insert(HasDocking::default())
                .id();

            let fleet_id = world
                .spawn_empty()
                .insert(ActionActive(Action::Undock))
                .insert(ActionUndock)
                .insert(LocationDocked {
                    parent_id: station_id,
                })
                .id();

            // update station docking
            world
                .get_mut::<HasDocking>(station_id)
                .unwrap()
                .docked
                .push(fleet_id);

            (fleet_id, station_id)
        });

        // check fleet
        assert!(world.get::<ActionActive>(fleet_id).is_none());
        assert!(world.get::<ActionUndock>(fleet_id).is_none());
        match world.get::<LocationSpace>(fleet_id) {
            Some(LocationSpace { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }

        // check docked ship was removed from docking
        assert!(world
            .get::<HasDocking>(station_id)
            .unwrap()
            .docked
            .is_empty());
    }

    #[test]
    fn test_undock_system_should_ignore_undock_if_not_docked() {
        let (world, entity) = test_system(system_undock, |world| {
            let sector_id = world.spawn_empty().id();

            let entity = world
                .spawn_empty()
                .insert(ActionActive(Action::Undock))
                .insert(ActionUndock)
                .insert(LocationSpace {
                    pos: Position::new(0.0, 0.0),
                    sector_id,
                })
                .id();

            entity
        });

        assert!(world.get::<ActionActive>(entity).is_none());
        assert!(world.get::<ActionUndock>(entity).is_none());
        match world.get::<LocationSpace>(entity) {
            Some(LocationSpace { pos, .. }) => {
                assert_v2(*pos, Position::new(0.0, 0.0));
            }
            _ => panic!(),
        }
    }
}
