//// extern crate space_lib;
//#[macro_export]
//macro_rules! get_or_continue {
//    ($res:expr) => {
//        match $res {
//            Some(val) => val,
//            None => {
//                continue;
//            }
//        }
//    };
//}
//
//#[macro_export]
//macro_rules! get_or_return {
//    ($res:expr) => {
//        match $res {
//            Some(val) => val,
//            None => {
//                return;
//            }
//        }
//    };
//}

#![allow(warnings)]
extern crate space_lib;

fn main() {
//    local_game::run();
    space_lib::test_combat::run();

    println!("done")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        assert!(true);
    }
}
