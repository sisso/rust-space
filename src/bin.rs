extern crate space_lib;

use space_lib::process;

fn main() {
    println!("{}", process(3));
//    println!("test")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        assert!(true);
    }
}
