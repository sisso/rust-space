// ----------------------------------------------------

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

// ----------------------------------------------------

#[macro_export]
macro_rules! debug {
    ($ctx:expr, $msg:expr) => (
        debugf!("DEBUG {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        debugf!("DEBUG {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}

#[macro_export]
macro_rules! info {
    ($ctx:expr, $msg:expr) => (
        debugf!("INFO {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        debugf!("INFO {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}

#[macro_export]
macro_rules! warn {
    ($ctx:expr, $msg:expr) => (
        debugf!("WARN {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        debugf!("WARN {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}

// ----------------------------------------------------

#[macro_export]
macro_rules! debugf {
    () => (debugf!(""));
    ($fmt:expr) => (match ::std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/debug.log") {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(format!("{}\n", $fmt).as_bytes()).ok();
            }
            Err(_) => {
                panic!("failed to open log file")
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
