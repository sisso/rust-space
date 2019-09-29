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
        println!("DEBUG {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        println!("DEBUG {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}

#[macro_export]
macro_rules! info {
    ($ctx:expr, $msg:expr) => (
        println!("INFO {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        println!("INFO {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}

#[macro_export]
macro_rules! warn {
    ($ctx:expr, $msg:expr) => (
        println!("WARN {location} - {ctx} - {msg}",
             location=file!(),
             ctx=$ctx,
             msg=$msg);
    );
    ($ctx:expr, $fmt:expr, $($arg:tt)*) => (
        println!("WARN {location} - {ctx} - {msg}",
                 location=file!(),
                 ctx=$ctx,
                 msg=format!($fmt, $($arg)*).as_str());
    );
}
