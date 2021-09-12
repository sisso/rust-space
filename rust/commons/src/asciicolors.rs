pub const RESET: &str = "\x1B[0m";
pub const COLOR_RED: u8 = 9;

pub fn fg(fg: u8) -> String {
    format!("\x1B[38;5;{}m", fg)
}

pub fn bg(bg: u8) -> String {
    format!("\x1B[48;5;{}m", bg)
}
