use specs::prelude::*;
use crate::game::extractables::Extractable;
use crate::game::wares::Cargo;
use crate::utils::{TotalTime, DeltaTime};
use crate::game::actions::{ActionExtract, ActionProgress, ActionActive, Action};
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
       trace!("running");

        let delta = data.delta_time.clone();
        let mut extract_complete = Vec::<Entity>::new();

        for (
            entity,
            active_action,
            _,
            cargo,
        ) in (
            &*data.entities,
            &data.action_active,
            &data.action_extract,
            &mut data.cargo,
        ).join() {
            let amount_extracted = delta.as_f32();

            let ware_id = match &active_action.0 {
               Action::Extract { target_id, ware_id } => *ware_id,
               other => panic!("{:?} unexpected action type {:?}", entity, active_action),
            };

            let amount_added = cargo.add_to_max(ware_id, amount_extracted);
            trace!("{:?} extracted {:?} {:?}, cargo now is {:?}/{:?}", entity, amount_extracted, ware_id, cargo.get_current(), cargo.get_total());
            if amount_added < amount_extracted {
                debug!("{:?} cargo is full, stopping to extract", entity);
                extract_complete.push(entity);
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
    use super::*;
    use super::super::*;
    use crate::test::test_system;
    use crate::utils::{DeltaTime};

    const WARE0: WareId = WareId(0);

    #[test]
    fn should_extract_ware() {
        let (world, entity) = test_system(ActionExtractSystem, |world| {
            world.insert(DeltaTime(1.0));

            let asteroid_id = world.create_entity()
                .build();

            world.create_entity()
                .with(ActionActive(Action::Extract { target_id: asteroid_id, ware_id: WARE0, }))
                .with(ActionExtract {})
                .with(Cargo::new(10.0))
                .build()
        });

        let cargo_storage = world.read_storage::<Cargo>();
        let cargo = cargo_storage.get(entity).unwrap();
        assert_eq!(1.0, cargo.get_amount(WARE0));
    }

    #[test]
    fn should_remove_action_when_cargo_is_full() {
        let (world, entity) = test_system(ActionExtractSystem, |world| {
            world.insert(DeltaTime(1.0));

            let asteroid_id = world.create_entity()
                .build();

            let mut cargo = Cargo::new(10.0);
            cargo.add(WARE0, 9.5).unwrap();

            world.create_entity()
                .with(ActionActive(Action::Extract { target_id: asteroid_id, ware_id: WARE0, }))
                .with(ActionExtract {})
                .with(cargo)
                .build()
        });

        let cargo_storage = world.read_storage::<Cargo>();
        let cargo = cargo_storage.get(entity).unwrap();
        assert_eq!(10.0, cargo.get_amount(WARE0));

        assert!(world.read_storage::<ActionActive>().get(entity).is_none());
        assert!(world.read_storage::<ActionExtract>().get(entity).is_none());
    }
}

