pub mod asciicolors;
pub mod csv;
pub mod grid;
pub mod hocon;
pub mod lineboundbox;
pub mod math;
pub mod prob;
pub mod random_grid;
pub mod recti;
pub mod tree;
pub mod v2i;

#[macro_export]
macro_rules! unwrap_or_continue {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => continue,
        }
    };
}
