//#![allow(warnings)]

#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;

pub mod ffi;
pub mod game;
pub mod specs_extras;
pub mod test;
pub mod utils;
//pub mod local_game;
//pub mod test_combat;
pub mod space_inputs_generated;
pub mod space_outputs_generated;

pub use space_galaxy;
