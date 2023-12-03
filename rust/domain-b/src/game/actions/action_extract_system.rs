use crate::game::actions::{Action, ActionActive, ActionExtract};
use crate::game::extractables::Extractable;
use crate::game::wares::Cargo;
use crate::utils::DeltaTime;

use bevy_ecs::prelude::*;
use std::borrow::BorrowMut;

pub struct ActionExtractSystem;

#[derive(SystemData)]
pub struct ActionExtractData<'a> {
    entities: Entities<'a>,
    delta_time: Read<'a, DeltaTime>,
    extractables: ReadStorage<'a, Extractable>,
    cargo: WriteStorage<'a, Cargo>,
    action_active: WriteStorage<'a, ActionActive>,
    action_extract: WriteStorage<'a, ActionExtract>,
}

impl<'a> System<'a> for ActionExtractSystem {
    type SystemData = ActionExtractData<'a>;

    fn run(&mut self, mut data: ActionExtractData) {
        log::trace!("running");

        let mut extract_complete = Vec::<Entity>::new();

        for (obj_id, active_action, action_extract, cargo) in (
            &*data.entities,
            &data.action_active,
            &mut data.action_extract,
            &mut data.cargo,
        )
            .join()
        {
            let (ware_id, target_id) = match &active_action.0 {
                Action::Extract { target_id, ware_id } => (*ware_id, *target_id),
                _other => {
                    log::warn!("{:?} unexpected action type {:?}", obj_id, active_action);
                    continue;
                }
            };

            let extractable = if let Some(extractable) = data.extractables.get(target_id) {
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
            let production =
                data.delta_time.as_f32() * extractable.accessibility + previous_rest_acc;

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

            let amount_added = cargo.add_to_max(ware_id, amount_extracted as u32);
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
                extract_complete.push(obj_id);
            }
        }

        for e in extract_complete {
            data.action_extract.borrow_mut().remove(e);
            data.action_active.borrow_mut().remove(e);
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::game::wares::Volume;
    use crate::test::TestSystemRunner;
    use crate::utils::DeltaTime;

    #[test]
    fn test_extraction() {
        let mut ts = TestSystemRunner::new(ActionExtractSystem);

        let ware_id = ts.world.create_entity().build();

        let asteroid_id = ts
            .world
            .create_entity()
            .with(Extractable {
                ware_id,
                accessibility: 1.0,
            })
            .build();

        let fleet_id = ts
            .world
            .create_entity()
            .with(ActionActive(Action::Extract {
                target_id: asteroid_id,
                ware_id,
            }))
            .with(ActionExtract::default())
            .with(Cargo::new(5))
            .build();

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
        assert!(world.read_storage::<ActionActive>().get(fleet_id).is_some());
        assert!(world
            .read_storage::<ActionExtract>()
            .get(fleet_id)
            .is_some());
    }

    fn assert_cargo(world: &World, fleet_id: ObjId, amount: Volume) {
        let current = world
            .read_storage::<Cargo>()
            .get(fleet_id)
            .expect("fail to find cargo")
            .get_current_volume();
        assert_eq!(current, amount);
    }

    fn assert_complete(world: &World, fleet_id: ObjId) {
        assert!(world.read_storage::<ActionActive>().get(fleet_id).is_none());
        assert!(world
            .read_storage::<ActionExtract>()
            .get(fleet_id)
            .is_none());
    }
}
