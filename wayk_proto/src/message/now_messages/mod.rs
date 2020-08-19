// ****** Now Messages ****** //

pub mod access_control;
pub mod input;
pub mod mouse;
pub mod sharing;
pub mod surface;
pub mod system;
pub mod update;

// re-export
pub use access_control::*;
pub use input::*;
pub use mouse::*;
pub use sharing::*;
pub use surface::*;
pub use system::*;
pub use update::*;

/*NOW_VIRTUAL_KEYBOARD CONSTANTS*/
pub const NOW_VKCODE_EXT: u16 = 0x0100;
pub const NOW_VKCODE_MASK: u16 = 0x00FF;
