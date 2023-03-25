mod game_api;
mod graphics;
mod main_gui;
pub mod state;
mod utils;

use godot::prelude::*;

struct SpaceGame;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceGame {}
