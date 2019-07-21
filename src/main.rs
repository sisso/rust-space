extern crate space_lib;

use space_lib::*;
mod game;
mod utils;

fn main() {
    let game = game::Game::new();

    //    println!("{}", process(3));
//    println!("test")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        assert!(true);
    }
}
