mod events;
mod game_api;
mod godot_utils;
mod graphics;
mod sector_view;
mod utils;

use godot::prelude::*;

struct SpaceApi;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceApi {}
