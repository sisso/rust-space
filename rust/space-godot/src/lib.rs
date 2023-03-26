mod game_api;
mod graphics;
mod main_gui;
pub mod state;
mod utils;
mod sector_view;

use godot::prelude::*;

struct SpaceGame;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceGame {}
