use super::sectors::*;
use super::Tick;
use super::action::*;
use super::objects::*;
use crate::utils::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Command {
    Idle,
    Mine,
}

struct MineState {
    target_obj_id: ObjId,
}

enum NavigationStateStep {
    MoveTo { pos: Position, },
    Jump {}
}

struct NavigationState {
    target_obj_id: ObjId,
    target_sector_id: SectorId,
    target_position: V2,
    path: Vec<NavigationStateStep>
}

struct State {
    mine: Option<MineState>,
    navigation: Option<NavigationState>,
}

impl State {
    fn new() -> Self {
        State {
            mine: None,
            navigation: None
        }
    }
}

// TODO: how to remove state on entity removal?
pub struct Commands {
    state: HashMap<ObjId, State>
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            state: HashMap::new()
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
                    let state = self.get_state(&obj.id);

                    if state.mine.is_none() {
                        // TODO: unwarp
                        let target = self.search_mine_target(objects, sectors, obj).unwrap();
                        // TODO: unwarp
                        let navigation = self.find_navigation_to(objects, sectors, &obj.location.as_space(), target.id).unwrap();

                        // FIXME: how to remove this double find?
                        let mut state = self.get_state_mut(obj.id);
                        state.mine = Some(MineState { target_obj_id: target.id });
                        state.navigation = Some(navigation);
                    }

                    // TODO: serious? 3 time get? what is your problem man
                    let state = self.get_state(&obj.id);
                    let action = self.navigation_next_action(sectors, obj, &state.navigation.as_ref().unwrap());
                    set_actions.push((obj.id, action));

//                    let state_mine = state.mine.as_ref().unwrap();
//
//                    if *sector_id == state_mine.target_sector_id {
//                        /*
//                        if near { mine } else { move_to }
//                        */
//                        // move to position
////                        set_actions.push((obj.id, Action::Fly { to: state_mine.target_position }));
//                    } else {
//                        /*
//                        if near { jump } else { move_to }
//                        */
//                        // jump to
////                        set_actions.push((obj.id, Action::Fly { to: jump_position }));
//                    }
                },
                (Command::Mine, Action::Fly { to}, Location::Space { sector_id, pos}) => {
                    // ignore
                },
                (Command::Mine, Action::Undock, Location::Docked { .. }) => {
                    // ignore
                },
                (Command::Idle, Action::Idle, _) => {
                    // ignore
                },
                (Command::Idle, _, _) => {
                    set_actions.push((obj.id, Action::Idle));
                },
                (a, b, c) => {
                    Log::warn("command", &format!("unknown {:?}", obj));
                }
            }
        }

        for (obj_id, action) in set_actions {
            objects.set_action(obj_id, action);
        }
    }

    fn get_state_mut(&mut self, id: ObjId) -> &mut State {
        self.state.entry(id).or_insert(State::new())
    }

    fn get_state(&self, id: &ObjId) -> &State {
        self.state.get(&id).unwrap()
    }

    fn search_mine_target<'a>(&self, objects: &'a ObjRepo, sectors: &SectorRepo, obj: &Obj) -> Option<&'a Obj> {
        // search minerable
        let candidates =
            objects.list().find(|obj| {
                obj.extractable.is_some()
            });

        // collect params
        candidates
    }

    // TODO: support moveable objects
    // TODO: support docked objects
    fn find_navigation_to(&self, objects: &ObjRepo, sectors: &SectorRepo, from: &LocationSpace, to_obj_id: ObjId) -> Option<NavigationState> {
        // collect params
        let target_pos= objects.get(&to_obj_id).location.as_space();

        let path = self.find_path(sectors, &from, &target_pos);

        Some(
            NavigationState {
                target_obj_id: to_obj_id,
                target_sector_id: target_pos.sector_id,
                target_position: target_pos.pos,
                path: vec![]
            }
        )
    }

    fn find_path(&self, sectors: &SectorRepo, from: &LocationSpace, to: &LocationSpace) -> Vec<NavigationStateStep> {
        vec![]
    }

    fn navigation_next_action(&self, sectors: &SectorRepo, obj: &Obj, navigation: &NavigationState) -> Action {
        unimplemented!()
    }
}

