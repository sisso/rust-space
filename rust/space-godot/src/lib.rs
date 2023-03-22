mod game_api;
mod main_gui;
pub mod state;

use commons::math::{Transform2, P2, V2};
use godot::engine::Engine;
use godot::prelude::*;
use godot::private::You_forgot_the_attribute__godot_api;
use space_domain::game::{scenery_random, Game};
use specs::Entity;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct SpaceGame;

#[gdextension]
unsafe impl ExtensionLibrary for SpaceGame {}
