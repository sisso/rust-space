use crate::game::actions::*;
use crate::game::objects::ObjId;
use crate::game::commands::*;
use crate::utils::*;

pub fn execute(commands: &mut Commands, actions: &mut Actions) {
    for (obj_id, state) in commands.list_mut() {
        match state.command {
            Command::Idle => {
                let action = actions.get_action(obj_id);
                match action {
                    Action::Idle => {
                        // ignore
                    },
                    other => {
                        info!("command", &format!("{:?} setting idle action", obj_id));
                        actions.set_action(obj_id, Action::Idle);
                    }
                }
            },
            _ => {},
        }
    }
}
