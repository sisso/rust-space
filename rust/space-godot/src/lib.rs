mod game_api;
mod game_api_runtime;
mod graphics;
mod main_gui;
mod sector_view;
mod utils;

use godot::prelude::*;

struct SpaceApi;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceApi {}
