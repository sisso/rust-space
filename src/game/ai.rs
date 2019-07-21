use super::sectors::*;
use super::Tick;
use super::command::*;
use super::action::*;
use super::objects::*;
use crate::utils::*;

pub struct Ai {

}

impl Ai {
    pub fn new() -> Self {
        Ai {

        }
    }

    pub fn tick(&mut self, tick: &Tick, objects: &mut ObjRepo, sectors: &SectorRepo) {
        let mut set_actions = vec![];

        for obj in objects.list() {
            match (&obj.command, &obj.action, &obj.location) {
                (Command::Mine, Action::Idle, Location::Docked { obj_id }) => {
                    set_actions.push((obj.id, Action::Undock));
                },
                (Command::Mine, Action::Idle, Location::Space { sector_id, pos}) => {
                    // check to mine, jump or dock
                },
                (Command::Mine, Action::Fly { to}, Location::Space { sector_id, pos}) => {
                    // ignore
                },
                (Command::Idle, Action::Idle, _) => {
                    // ignore
                },
                (Command::Idle, _, _) => {
                    set_actions.push((obj.id, Action::Idle));
                },
                (a, b, c) => {
                    Log::warn("ai", &format!("unknown {:?} {:?} {:?}", a, b, c));
                }
            }
        }

        for (obj_id, action) in set_actions {
            objects.set_action(obj_id, action);
        }
    }
}

