pub mod asciicolors;
pub mod csv;
pub mod hocon;
pub mod lineboundbox;
pub mod math;
pub mod prob;
pub mod tree;

#[macro_export]
macro_rules! unwrap_or_continue {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => continue,
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => return,
        }
    };
}
