mod game;

pub fn process_2(v: u32) -> u32 {
    v * 2
}

#[no_mangle]
pub extern fn process(v: u32) -> u32 {
    v * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert!(6 == process(3))
    }
}
