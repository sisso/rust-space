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

#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        eprintln!("{} - {} - {}",
            $target,
            $lvl,
            format_args!($($arg)*));
    });
    ($lvl:expr, $($arg:tt)+) => (log!(target: file!(), $lvl, $($arg)+))
}

///
/// Hacked from https://docs.rs/log/0.4.8/src/log/macros.rs.html#135-142
///

/// Logs a message at the error level.
///
/// # Examples
///
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, "ERROR", $($arg)+);
    );
    ($($arg:tt)+) => (
        log!("ERROR", $($arg)+);
    )
}

/// Logs a message at the warn level.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, "WARN", $($arg)+);
    );
    ($($arg:tt)+) => (
        log!("WARN", $($arg)+);
    )
}

/// Logs a message at the info level.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, "INFO", $($arg)+);
    );
    ($($arg:tt)+) => (
        log!("INFO", $($arg)+);
    )
}

/// Logs a message at the debug level.
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, "DEBUG", $($arg)+);
    );
    ($($arg:tt)+) => (
        log!("DEBUG", $($arg)+);
    )
}

/// Logs a message at the trace level.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        // log!(target: $target, "TRACE", $($arg)+);
    );
    ($($arg:tt)+) => (
        // log!("TRACE", $($arg)+);
    )
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
