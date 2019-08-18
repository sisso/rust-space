// extern crate space_lib;
#[macro_export]
macro_rules! get_or_continue {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                continue;
            }
        }
    };
}

#[macro_export]
macro_rules! get_or_return {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                return;
            }
        }
    };
}

mod game;
mod utils;
mod local_game;
mod test_combat;

fn main() {
//    local_game::run();
    test_combat::run();

    println!("done")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        assert!(true);
    }
}
