// Exec

#[derive(Encode, Decode, Debug, Clone, Copy)]
pub enum ExecMessageType {
    #[value = 0x00]
    CapsetReq,
    #[fallback]
    Other(u8),
}
