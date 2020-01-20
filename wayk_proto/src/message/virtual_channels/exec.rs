// Exec

use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ExecMessageType {
    CapsetReq = 0x00,
    // TODO: ExecMessageType enum
}
