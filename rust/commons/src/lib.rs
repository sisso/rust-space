use std::time::{Duration, Instant};

pub mod asciicolors;
pub mod csv;
pub mod grid;
pub mod hocon;
pub mod lineboundbox;
pub mod math;
pub mod prob;
pub mod random_grid;
pub mod recti;
pub mod tree;
pub mod v2i;

#[macro_export]
macro_rules! unwrap_or_continue {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => continue,
        }
    };
}

pub struct TimeDeadline(Instant);

impl TimeDeadline {
    pub fn new(max: Duration) -> Self {
        TimeDeadline(Instant::now() + max)
    }

    pub fn is_timeout(&self) -> bool {
        Instant::now() >= self.0
    }
}

#[test]
fn test_deadline() {
    let deadline = TimeDeadline::new(Duration::from_micros(90));
    assert!(!deadline.is_timeout());
    std::thread::sleep(Duration::from_micros(100));
    assert!(deadline.is_timeout());
}

#[macro_export]
macro_rules! debugf {
    () => (debugf!(""));
    ($fmt:expr) => (match ::std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/debug.log") {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(format!("{} {} {}\n", $fmt, line!(), file!()).as_bytes()).ok();
            }
            Err(e) => {
                panic!("failed to open log file: {:?}", e)
            },
        });
    ($fmt:expr, $($arg:tt)*) => (match ::std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/debug.log") {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(format!(concat!($fmt, "\n"), $($arg)*).as_bytes()).ok();
            }
            Err(_) => {
                panic!("failed to open log file")
            },
        });
}
