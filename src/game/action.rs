use crate::utils::{V2, Position, Log};
use crate::game::objects::{ObjRepo, Location};
use crate::game::sectors::SectorRepo;
use crate::game::Tick;

#[derive(Clone,Debug)]
pub enum Action {
    Idle,
    Undock,
    Fly { to: Position },
    Jump,
    Mine,
}

pub struct Actions {

}

impl Actions {
    pub fn new() -> Self {
        Actions {}
    }

    pub fn tick(&mut self, tick: &Tick, objects: &mut ObjRepo, sectors: &SectorRepo) {
        let mut set_actions = vec![];

        for obj in objects.list() {
            match (&obj.action, &obj.location) {
                (Action::Idle, _) => {
                    // ignore
                },
                (Action::Undock, Location::Docked { obj_id }) => {
                    let station = objects.get(&obj_id);

                    let (sector_id, pos) = match station.location {
                        Location::Space { sector_id, pos } => (sector_id, pos),
                        _ => panic!("station is not at space")
                    };

                    let new_location = Location::Space {
                        sector_id,
                        pos
                    };

                    set_actions.push((obj.id, Action::Idle, Some(new_location)));
                },
                (Action::Undock, Location::Space { .. }) => {
                    set_actions.push((obj.id, Action::Idle, None));
                },
                _ => {
                    Log::warn("actions", &format!("unknown {:?}", obj));
                }
            }
        }

        for (obj_id, action, location) in set_actions {
            objects.set_action(obj_id, action);

            if let Some(location) = location {
                objects.set_location(obj_id, location);
            }
        }
    }
}
