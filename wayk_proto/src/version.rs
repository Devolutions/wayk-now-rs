use lazy_static::lazy_static;

pub const WAYK_NOW_VERSION_MAJOR: u8 = 3;
pub const WAYK_NOW_VERSION_MINOR: u8 = 3;
pub const WAYK_NOW_VERSION_PATCH: u8 = 1;
pub const WAYK_NOW_NAME_STRING: &str = "Wayk Now";

lazy_static! {
    pub static ref WAYK_NOW_VERSION_STRING: String = format!(
        "{}.{}.{}",
        WAYK_NOW_VERSION_MAJOR, WAYK_NOW_VERSION_MINOR, WAYK_NOW_VERSION_PATCH
    );
    pub static ref WAYK_NOW_VERSION: [u16; 3] = [
        u16::from(WAYK_NOW_VERSION_MAJOR) * 1000,
        u16::from(WAYK_NOW_VERSION_MINOR) * 100,
        u16::from(WAYK_NOW_VERSION_PATCH)
    ];
}
