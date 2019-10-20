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
        println!("{} - {} - {}",
            $target,
            $lvl,
            format_args!($($arg)*));
    });
    ($lvl:expr, $($arg:tt)+) => (log!(target: module_path!(), $lvl, $($arg)+))
}

///
/// Hacked from https://docs.rs/log/0.4.8/src/log/macros.rs.html#135-142
///

/// Logs a message at the error level.
///
/// # Examples
///
/// ```edition2018
/// use log::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// error!(target: "app_events", "App Error: {}, Port: {}", err_info, 22);
/// # }
/// ```
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
///
/// # Examples
///
/// ```edition2018
/// use log::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// warn!(target: "input_events", "App received warning: {}", warn_description);
/// # }
/// ```
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
///
/// # Examples
///
/// ```edition2018
/// use log::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// info!(target: "connection_events", "Successfull connection, port: {}, speed: {}",
///       conn_info.port, conn_info.speed);
/// # }
/// ```
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
///
/// # Examples
///
/// ```edition2018
/// use log::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// debug!(target: "app_events", "New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
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
///
/// # Examples
///
/// ```edition2018
/// use log::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// trace!(target: "app_events", "x is {} and y is {}",
///        if pos.x >= 0.0 { "positive" } else { "negative" },
///        if pos.y >= 0.0 { "positive" } else { "negative" });
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, "TRACE", $($arg)+);
    );
    ($($arg:tt)+) => (
        log!("TRACE", $($arg)+);
    )
}
