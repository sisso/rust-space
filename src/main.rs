fn main() {
    println!("{}", process(3));
}

#[cfg(test)]
mod tests {
    use super::*;
    use space::process;

    #[test]
    fn test1() {
        assert!(6 == process(3))
    }
}
