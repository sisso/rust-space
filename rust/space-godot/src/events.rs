use godot::prelude::*;

#[derive(Default, Debug, Clone)]
pub struct GameEvent {
    pub target_id: i64,
    pub added: bool,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct EventsList {
    events: Vec<GameEvent>,
}

#[godot_api]
impl IRefCounted for EventsList {}

#[godot_api]
impl EventsList {
    pub fn from_vec(events: Vec<GameEvent>) -> Gd<EventsList> {
        Gd::from_init_fn(|base| Self { events })
    }

    #[func]
    pub fn len(&self) -> i32 {
        self.events.len() as i32
    }

    #[func]
    pub fn get_entity(&self, i: i32) -> i64 {
        self.events[i as usize].target_id
    }

    #[func]
    pub fn is_add(&self, i: i32) -> bool {
        self.events[i as usize].added
    }
}
