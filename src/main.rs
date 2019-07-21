// extern crate space_lib;

//use space_lib::*;
mod game;
mod utils;
mod local_game;

fn main() {
    local_game::run();
    println!("done")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        assert!(true);
    }
}
