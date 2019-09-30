use crate::game::Game;
use std::time::Duration;
use crate::utils::Seconds;
use crate::game::events::EventKind;
use crate::game::locations::Location;

pub struct GameApi {
    game: Game,
    total_time: f32,
}

/// Represent same interface we intend to use through FFI
impl GameApi {
    pub fn new() -> Self {
        GameApi {
            game: Game::new(),
            total_time: 0.0
        }
    }

    pub fn new_game(&mut self) {
        crate::local_game::init_new_game(&mut self.game);
    }

    pub fn update(&mut self, elapsed: Duration) {
        let delta = (elapsed.as_millis() as f32) / 1000.0;
        self.total_time += delta;
        self.game.tick(Seconds(self.total_time), Seconds(delta))
    }

    /// TODO: remove this method, should not be used directly
    pub fn get_game(&self) -> &Game {
        &self.game
    }

    pub fn set_inputs(&mut self, bytes: &Vec<u8>) -> bool {
        false
    }

    pub fn get_inputs<F>(&mut self, f: F) -> bool where F: FnOnce(Vec<u8>) {
        info!("game_api", "get_inputs");
        let events = self.game.events.take();

        for event in events {
            match event.kind {
                EventKind::Add => {
                    let kind =
                        if self.game.extractables.get_extractable(&event.id).is_some() {
                            "asteroid"
                        } else if self.game.objects.get(&event.id).has_dock {
                            "station"
                        } else {
                            "fleet"
                        };

                    match self.game.locations.get_location(&event.id) {
                        Some(Location::Space { sector_id, pos} ) => {
                            info!("game_api", "{:?} {:?} added at {:?}/{:?}", kind, event, sector_id, pos);
                        },
                        Some(Location::Docked { docked_id } ) => {
                            let docked_location = self.game.locations.get_location(docked_id).unwrap().get_space();
                            info!("game_api", "{:?} {:?} added and docked at {:?}/{:?}", kind, event, docked_location.sector_id, docked_location.pos);
                        },
                        None => {
                            warn!("game_api", "Added {:?}, but has no location", event);
                        }
                    }
                }
                EventKind::Jump => {
                    let location = self.game.locations.get_location(&event.id).unwrap().get_space();
                    info!("game_api", "{:?} jump to {:?}/{:?}", event, location.sector_id, location.pos);
                },
                EventKind::Move => {
                    let location = self.game.locations.get_location(&event.id).unwrap().get_space();
                    info!("game_api", "{:?} move to {:?}", event, location.pos);
                }
            }
        }
        false
    }
}

