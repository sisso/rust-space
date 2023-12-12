use crate::game::actions::{Action, ActionActive, ActionExtract};
use crate::game::extractables::Extractable;
use crate::game::utils::DeltaTime;
use crate::game::wares::Cargo;

use bevy_ecs::prelude::*;

pub fn system_extract(
    mut commands: Commands,
    delta_time: Res<DeltaTime>,
    mut query: Query<(Entity, &ActionActive, &mut ActionExtract, &mut Cargo)>,
    query_extractables: Query<&Extractable>,
) {
    log::trace!("running");
    let delta_time = *delta_time;

    for (obj_id, active_action, mut action_extract, mut cargo) in &mut query {
        let (ware_id, target_id) = match &active_action.0 {
            Action::Extract { target_id, ware_id } => (*ware_id, *target_id),
            _other => {
                log::warn!("{:?} unexpected action type {:?}", obj_id, active_action);
                continue;
            }
        };

        let extractable = if let Some(extractable) = query_extractables.get(target_id).ok() {
            extractable
        } else {
            log::warn!(
                "{:?} try to extract {:?} that is not extractable",
                obj_id,
                target_id
            );
            continue;
        };

        if extractable.ware_id != ware_id {
            log::warn!(
                "{:?} try to extract ware_id {:?} from {:?} but it can only produce {:?}",
                obj_id,
                ware_id,
                target_id,
                extractable.ware_id
            );
            continue;
        }

        let previous_rest_acc = action_extract.rest_acc;
        let production = delta_time.as_f32() * extractable.accessibility + previous_rest_acc;

        let amount_extracted = production.floor();
        action_extract.rest_acc = production - amount_extracted;
        let amount_extracted = amount_extracted as u32;

        let ware_id = match &active_action.0 {
            Action::Extract {
                target_id: _,
                ware_id,
            } => *ware_id,
            _other => panic!("{:?} unexpected action type {:?}", obj_id, active_action),
        };

        let amount_added = cargo.add_to_max(ware_id, amount_extracted);
        log::trace!(
                "{:?} extracted {:?}, acc {:?}, total extracted {:?} with volume of {:?} and rest of {:?}, cargo now is {:?}/{:?}",
                obj_id,
                ware_id,
                previous_rest_acc,
                production,
                amount_extracted,
                action_extract.rest_acc,
                cargo.get_current_volume(),
                cargo.get_max(),
            );
        if amount_added < amount_extracted {
            log::debug!("{:?} cargo is full, stopping to extract", obj_id);
            commands
                .entity(obj_id)
                .remove::<ActionExtract>()
                .remove::<ActionActive>();
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::game::utils::DeltaTime;
    use crate::game::wares::Volume;
    use crate::test::{init_trace_log, TestSystemRunner};

    #[test]
    fn test_extraction() {
        init_trace_log().unwrap();
        let mut ts = TestSystemRunner::new(system_extract);

        let ware_id = ts.world.spawn_empty().id();

        let asteroid_id = ts
            .world
            .spawn_empty()
            .insert(Extractable {
                ware_id,
                accessibility: 1.0,
            })
            .id();

        let fleet_id = ts
            .world
            .spawn_empty()
            .insert(ActionActive(Action::Extract {
                target_id: asteroid_id,
                ware_id,
            }))
            .insert(ActionExtract::default())
            .insert(Cargo::new(5))
            .id();

        // first tick, it should be not enough to generate one resource
        ts.tick_timed(DeltaTime(0.5));
        assert_running(&ts.world, fleet_id);
        assert_cargo(&ts.world, fleet_id, 0);

        // second tick, should be enough to generate a cargo
        ts.tick_timed(DeltaTime(0.5));
        assert_running(&ts.world, fleet_id);
        assert_cargo(&ts.world, fleet_id, 1);

        // ticket a big jump leap second, should fill all the cargo and action completed
        ts.tick_timed(DeltaTime(60.0));
        assert_cargo(&ts.world, fleet_id, 5);
        assert_complete(&ts.world, fleet_id);
    }

    fn assert_running(world: &World, fleet_id: ObjId) {
        assert!(world.get::<ActionActive>(fleet_id).is_some());
        assert!(world.get::<ActionExtract>(fleet_id).is_some());
    }

    fn assert_cargo(world: &World, fleet_id: ObjId, amount: Volume) {
        let current = world
            .get::<Cargo>(fleet_id)
            .expect("fail to find cargo")
            .get_current_volume();
        assert_eq!(current, amount);
    }

    fn assert_complete(world: &World, fleet_id: ObjId) {
        assert!(world.get::<ActionActive>(fleet_id).is_none());
        assert!(world.get::<ActionExtract>(fleet_id).is_none());
    }
}
