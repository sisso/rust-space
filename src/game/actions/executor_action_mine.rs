use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;
use crate::game::locations::*;
use crate::game::sectors::{Sector, SectorRepo};
use crate::game::Tick;
use crate::game::extractables::Extractables;
use crate::game::wares::Cargos;

pub fn execute(tick: &Tick,
               actions: &mut Actions,
               locations: &mut Locations,
               extractables: &Extractables,
               cargos: &mut Cargos) {

    for (obj_id, state) in actions.list_mut() {
        match (&state.action, &state.action_delay) {
            (Action::Mine { target }, Some(delay)) if delay.0 < tick.delta_time.0 => {
                let cargo = cargos.get_cargo(obj_id);
                let cargo = match cargo {
                    Some(cargo) => { cargo },
                    None => {
                        Log::warn("executor_action_mine", &format!("{:?} can not mine, has no cargo", obj_id));
                        continue;
                    }
                };

                let ext = extractables.get_extractable(&target);
                Log::debug("executor_action_mine", &format!("{:?} mine complete, extracted {:?}", obj_id, ext.ware_id));


                Log::debug("executor_action_mine", &format!("{:?} set mine time delay {:?}", obj_id, ext.time));
                state.action_delay = Some(ext.time);
            },
            (Action::Mine { target }, Some(delay)) => {
                state.action_delay = Some(Seconds(delay.0 - tick.delta_time.0));
            },
            (Action::Mine { target }, None) => {
                let ext = extractables.get_extractable(&target);
                Log::debug("executor_action_mine", &format!("{:?} set mine time delay {:?}", obj_id, ext.time));
                state.action_delay = Some(ext.time);
            },
            _ => {},
        }
    }
}
