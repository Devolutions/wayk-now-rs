// File Transfer

use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy)]
#[repr(u8)]
pub enum FileTransferMessageType {
    CapsetReq = 0x00,
    // TODO: FileTransferMessageType enum
}
