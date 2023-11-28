use crate::game::actions::{Action, ActionActive, ActionExtract};
use crate::game::extractables::Extractable;
use crate::game::wares::{Cargo, ResourceExtraction, Volume};
use crate::utils::DeltaTime;

use specs::prelude::*;
use std::borrow::BorrowMut;

const EXTRACTION_PER_SECOND: f32 = 100.0;

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
        let default_extraction_speed: ResourceExtraction = 1.0;

        for (obj_id, active_action, _, cargo) in (
            &*data.entities,
            &data.action_active,
            &data.action_extract,
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

            let production =
                data.delta_time.as_f32() * extractable.accessibility * default_extraction_speed;
            let amount_extracted: Volume = (production * EXTRACTION_PER_SECOND) as u32;

            let ware_id = match &active_action.0 {
                Action::Extract {
                    target_id: _,
                    ware_id,
                } => *ware_id,
                _other => panic!("{:?} unexpected action type {:?}", obj_id, active_action),
            };

            let amount_added = cargo.add_to_max(ware_id, amount_extracted);
            log::trace!(
                "{:?} extracted {:?} {:?}, cargo now is {:?}/{:?}",
                obj_id,
                amount_extracted,
                ware_id,
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
    use crate::test::test_system;
    use crate::utils::DeltaTime;

    #[test]
    fn should_extract_ware() {
        let (world, (entity, ware_id)) = test_system(ActionExtractSystem, |world| {
            world.insert(DeltaTime(1.0));

            let ware_id = world.create_entity().build();

            let asteroid_id = world
                .create_entity()
                .with(Extractable {
                    ware_id,
                    accessibility: 1.0,
                })
                .build();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Extract {
                    target_id: asteroid_id,
                    ware_id,
                }))
                .with(ActionExtract::default())
                .with(Cargo::new(100))
                .build();

            (entity, ware_id)
        });

        let cargo_storage = world.read_storage::<Cargo>();
        let cargo = cargo_storage.get(entity).unwrap();
        assert_eq!(100, cargo.get_amount(ware_id));
    }

    #[test]
    fn should_remove_action_when_cargo_is_full() {
        let (world, (entity, ware_id)) = test_system(ActionExtractSystem, |world| {
            world.insert(DeltaTime(1.0));

            let ware_id = world.create_entity().build();

            let asteroid_id = world
                .create_entity()
                .with(Extractable {
                    ware_id,
                    accessibility: 1.0,
                })
                .build();

            let mut cargo = Cargo::new(100);
            cargo.add(ware_id, 95).unwrap();

            let entity = world
                .create_entity()
                .with(ActionActive(Action::Extract {
                    target_id: asteroid_id,
                    ware_id,
                }))
                .with(ActionExtract::default())
                .with(cargo)
                .build();

            (entity, ware_id)
        });

        let cargo_storage = world.read_storage::<Cargo>();
        let cargo = cargo_storage.get(entity).unwrap();
        assert_eq!(100, cargo.get_amount(ware_id));

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionExtract>().get(entity).is_none());
    }
}
