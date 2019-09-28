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
