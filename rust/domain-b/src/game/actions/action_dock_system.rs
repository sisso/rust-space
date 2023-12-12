use bevy_ecs::prelude::*;

use super::super::locations::*;
use super::*;

use crate::game::events::{CommandSendEvent, EventKind, GEvent, GEvents};

use crate::game::dock::HasDocking;

/// dock object into a hasdock. There is no validation, it assume (wrongly) that ship can dock
pub fn system_dock(
    mut commands: Commands,
    mut query_hasdock: Query<&mut HasDocking>,
    query_action: Query<(Entity, &ActionActive), With<ActionDock>>,
) {
    log::trace!("running");

    for (obj_id, action) in &query_action {
        let target_id = match action.get_action() {
            Action::Dock { target_id } => target_id.clone(),
            _ => {
                commands.entity(obj_id).remove::<ActionDock>();
                continue;
            }
        };

        // update docked object (maybe move to a command?)
        query_hasdock
            .get_mut(target_id)
            .expect("target has no docking component")
            .docked
            .push(obj_id);

        log::debug!("{:?} docked at {:?}", obj_id, target_id);

        commands
            .entity(obj_id)
            .insert(LocationDocked {
                parent_id: target_id,
            })
            .remove::<LocationSpace>()
            .remove::<LocationOrbit>()
            .remove::<ActionActive>()
            .remove::<ActionDock>();

        commands.add(CommandSendEvent::from(GEvent::new(obj_id, EventKind::Dock)));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::dock::HasDocking;
    use bevy_ecs::system::RunSystemOnce;

    use crate::game::utils::Position;

    #[test]
    fn test_dock_system_should_dock() {
        let mut world = World::new();
        world.insert_resource(GEvents::default());

        let station_position = Position::ZERO;

        let sector_0 = world.spawn_empty().id();

        let station_id = world
            .spawn_empty()
            .insert(LocationSpace {
                pos: station_position,
                sector_id: sector_0,
            })
            .insert(HasDocking::default())
            .id();

        let fleet_id = world
            .spawn_empty()
            .insert(ActionActive(Action::Dock {
                target_id: station_id,
            }))
            .insert(ActionDock)
            .insert(LocationSpace {
                pos: station_position,
                sector_id: sector_0,
            })
            .id();

        world.run_system_once(system_dock);

        // check
        assert!(world.get::<ActionActive>(fleet_id).is_none());
        assert!(world.get::<ActionDock>(fleet_id).is_none());
        match world.get::<LocationDocked>(fleet_id) {
            Some(LocationDocked { parent_id }) => assert_eq!(*parent_id, station_id),
            _ => panic!(),
        }

        // check if docked object contain the new obj
        let station_has_dock = world.get::<HasDocking>(station_id).unwrap().clone();
        assert_eq!(1, station_has_dock.docked.len());
        assert_eq!(fleet_id, station_has_dock.docked[0]);
    }
}
