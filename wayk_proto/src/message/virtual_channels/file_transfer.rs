// File Transfer

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub enum FileTransferMessageType {
    #[value = 0x00]
    CapsetReq,
    #[fallback]
    Other(u8),
}
