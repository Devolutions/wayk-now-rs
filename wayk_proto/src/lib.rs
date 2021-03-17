// for later no_std support
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate wayk_proto_derive;

////////////////////////////////////////////////////////////////////////////////

#[macro_use]
#[doc(hidden)]
pub mod macros;

pub mod auth;
pub mod channels_manager;
pub mod container;
pub mod error;
pub mod header;
pub mod message;
pub mod packet;
pub mod serialization;
pub mod sharee;
pub mod sm;
pub mod version;

////////////////////////////////////////////////////////////////////////////////

extern crate self as wayk_proto;
extern crate static_assertions as sa;
