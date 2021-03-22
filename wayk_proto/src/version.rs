#![allow(clippy::identity_op)]

macro_rules! major {
    () => {
        21
    };
}
macro_rules! minor {
    () => {
        1
    };
}
macro_rules! patch {
    () => {
        0
    };
}

pub const WAYK_NOW_VERSION_MAJOR: u8 = major!();
pub const WAYK_NOW_VERSION_MINOR: u8 = minor!();
pub const WAYK_NOW_VERSION_PATCH: u8 = patch!();
pub const WAYK_NOW_NAME_STRING: &str = "Wayk Now";
pub const WAYK_NOW_VERSION_STRING: &str = concat!(major!(), ".", minor!(), ".", patch!());
pub const WAYK_NOW_VERSION: [u16; 3] = [major!() * 1000, minor!() * 100, patch!()];
