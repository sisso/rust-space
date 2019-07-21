pub struct Log {

}

impl Log {
    pub fn new() -> Self {
        Log {

        }
    }

    pub fn info(ctx: &str, s: &str) {
        println!("{} - {}", ctx, s);
    }
}
