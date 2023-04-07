mod game_api;
mod graphics;
mod main_gui;
mod runtime;
mod sector_view;
pub mod state;
mod utils;

use godot::prelude::*;

struct SpaceApi;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceApi {}
