extern crate space_domain;

#[no_mangle]
pub extern fn add_numbers(number1: i32, number2: i32) -> i32 {
    number1 + number2
}

#[no_mangle]
pub extern fn execute() {
    space_domain::test_combat::run();
}
