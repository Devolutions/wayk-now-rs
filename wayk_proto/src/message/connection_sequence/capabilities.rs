use crate::container::Vec8;
use crate::error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt, Result};
use crate::message::{MouseMode, NowString, NowString64, NowSurfaceListReqMsg, NowSystemOsInfo};
use crate::serialization::{Decode, Encode};
use byteorder::{LittleEndian, ReadBytesExt};
use core::convert::TryFrom;
use core::mem;
use std::io::{Cursor, Write};

// NOW_SURFACE_CAPSET

__flags_struct! {
    SurfaceCapsetFlags: u32 => {
        list_req = LIST_REQ = 0x0000_0001,
        select = SELECT = 0x0000_0002,
        multi = MULTI = 0x0000_0004,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct SurfaceCapset {
    pub flags: SurfaceCapsetFlags,
    pub list_req: NowSurfaceListReqMsg,
}

impl SurfaceCapset {
    const NAME: &'static str = "NowSurface";

    pub fn new(flags: SurfaceCapsetFlags, list_req: NowSurfaceListReqMsg) -> Self {
        Self { flags, list_req }
    }
}

// NOW_UPDATE_CAPSET

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum Codec {
    #[value = 0x0000]
    Unspecified,
    #[value = 0x0001]
    Thor,
    #[value = 0x0002]
    JPEG,
    #[value = 0x0003]
    GFWX,
    #[fallback]
    Other(u16),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum QualityMode {
    #[value = 0x00]
    Unspecified,
    #[value = 0x01]
    Low,
    #[value = 0x02]
    Medium,
    #[value = 0x03]
    High,
    #[fallback]
    Other(u8),
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct NowCodecDef {
    size: u16,
    pub id: Codec,
    pub flags: u32,
}

impl NowCodecDef {
    pub fn new(codec_id: Codec) -> Self {
        Self::new_with_flags(codec_id, 0)
    }

    pub fn new_with_flags(codec_id: Codec, flags: u32) -> Self {
        Self {
            size: Self::size() as u16,
            id: codec_id,
            flags,
        }
    }

    pub fn size() -> usize {
        8
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct UpdateCapset {
    flags: u32,
    pub quality_mode: QualityMode,
    padding: u8,
    pub codec_id: Codec,
    performance: u32,
    pub codecs: Vec8<NowCodecDef>,
}

impl UpdateCapset {
    const NAME: &'static str = "NowUpdate";

    pub fn new(quality_mode: QualityMode, codec_id: Codec) -> Self {
        Self {
            flags: 0,
            quality_mode,
            padding: 0,
            codec_id,
            performance: 0,
            codecs: Vec8(Vec::new()),
        }
    }

    pub fn new_with_supported_codecs(codecs: Vec<NowCodecDef>) -> Self {
        Self {
            flags: 0,
            quality_mode: QualityMode::Unspecified,
            padding: 0,
            codec_id: Codec::Unspecified,
            performance: 0,
            codecs: Vec8(codecs),
        }
    }
}

// NOW_INPUT_CAPSET

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum InputActionCode {
    #[value = 0x0001]
    SAS,
    #[value = 0x0010]
    ClipboardCut,
    #[value = 0x0011]
    ClipboardCopy,
    #[value = 0x0012]
    ClipboardPaste,
    #[value = 0x0013]
    ClipboardCopySpecial,
    #[value = 0x0014]
    ClipboardPasteSpecial,
    #[value = 0x0015]
    SelectAll,
    #[value = 0x0016]
    Undo,
    #[value = 0x0017]
    Redo,
    #[value = 0x0020]
    Shutdown,
    #[value = 0x0021]
    Reboot,
    #[value = 0x0022]
    RebootSafe,
    #[fallback]
    Other(u16),
}

__flags_struct! {
    InputActionFlags: u16 => {
        disabled = DISABLED = 0x0001,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowInputActionDef {
    pub code: InputActionCode,
    pub flags: InputActionFlags,
}

impl NowInputActionDef {
    pub fn new_enabled(code: InputActionCode) -> Self {
        Self {
            code,
            flags: InputActionFlags::new_empty(),
        }
    }

    pub fn new_disabled(code: InputActionCode) -> Self {
        Self {
            code,
            flags: InputActionFlags::new_empty().set_disabled(),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct InputCapset {
    flags: u32,
    reserved: u32,
    pub actions: Vec8<NowInputActionDef>,
}

impl InputCapset {
    const NAME: &'static str = "NowInput";

    pub fn new_with_actions(actions: Vec<NowInputActionDef>) -> Self {
        Self {
            flags: 0,
            reserved: 0,
            actions: Vec8(actions),
        }
    }
}

// NOW_MOUSE_CAPSET

__flags_struct! {
    MouseCapsetFlags: u32 => {
        large = LARGE = 0x0000_0001,
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct MouseCapset {
    pub flags: MouseCapsetFlags,
    pub mode: MouseMode,
    padding: u8,
    reserved: u16,
}

impl MouseCapset {
    const NAME: &'static str = "NowMouse";

    pub fn new(mode: MouseMode, flags: MouseCapsetFlags) -> Self {
        Self {
            flags,
            mode,
            padding: 0,
            reserved: 0,
        }
    }
}

// NOW_ACCESS_CAPSET

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum AccessControlCode {
    #[value = 0x0001]
    Viewing,
    #[value = 0x0002]
    Interact,
    #[value = 0x0003]
    Clipboard,
    #[value = 0x0004]
    FileTransfer,
    #[value = 0x0005]
    Exec,
    #[value = 0x0006]
    Chat,
    #[fallback]
    Other(u16),
}

__flags_struct! {
    AccessFlags: u16 => {
        allowed = ALLOWED = 0x0000_0001,
        confirm = CONFIRM = 0x0000_0002,
        disabled = DISABLED = 0x0000_0004,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct AccessControlDef {
    pub code: AccessControlCode,
    pub flags: AccessFlags,
}

impl AccessControlDef {
    pub fn new_with_flags(code: AccessControlCode, flags: AccessFlags) -> Self {
        Self { code, flags }
    }

    pub fn new_allowed(code: AccessControlCode) -> Self {
        Self::new_with_flags(code, AccessFlags::new_empty().set_allowed())
    }

    pub fn new_confirm(code: AccessControlCode) -> Self {
        Self::new_with_flags(code, AccessFlags::new_empty().set_confirm())
    }

    pub fn new_disabled(code: AccessControlCode) -> Self {
        Self::new_with_flags(code, AccessFlags::new_empty().set_disabled())
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct AccessCapset {
    flags: u32,
    reserved: u32,
    pub access_controls: Vec8<AccessControlDef>,
}

impl AccessCapset {
    const NAME: &'static str = "NowAccess";

    pub fn new_with_access_controls(access_controls: Vec<AccessControlDef>) -> Self {
        Self {
            flags: 0,
            reserved: 0,
            access_controls: Vec8(access_controls),
        }
    }
}

// NOW_LICENSE_CAPSET

__flags_struct! {
    LicenseCapsetFlags: u32 => {
        licensed = LICENSED = 0x0000_0001,
        mobile = MOBILE = 0x0000_0002,
        web = WEB = 0x0000_0004,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct LicenseCapset {
    pub flags: LicenseCapsetFlags,
}

impl LicenseCapset {
    const NAME: &'static str = "NowLicense";
}

// NOW_TRANSPORT_CAPSET

#[derive(Encode, Decode, Debug, Clone)]
pub struct TransportCapset {
    flags: u32,
}

impl Default for TransportCapset {
    fn default() -> Self {
        TransportCapset { flags: 0 }
    }
}

impl TransportCapset {
    const NAME: &'static str = "NowTransport";
}

// NOW_SYSTEM_CAPSET

__flags_struct! {
    SystemCapsetFlags: u32 => {
        os_info = OS_INFO = 0x0000_0001,
    }
}

#[derive(Debug, Clone)]
pub struct SystemCapset {
    pub flags: SystemCapsetFlags,
    pub os_info: Option<NowSystemOsInfo>,
}

impl SystemCapset {
    const NAME: &'static str = "NowSystem";

    pub fn new_os_info(os_info: NowSystemOsInfo) -> Self {
        Self {
            flags: SystemCapsetFlags::new_empty().set_os_info(),
            os_info: Some(os_info),
        }
    }
}

impl Encode for SystemCapset {
    fn encoded_len(&self) -> usize {
        let os_info_len = if let Some(os_info) = &self.os_info {
            os_info.encoded_len()
        } else {
            0
        };

        self.flags.encoded_len() + os_info_len
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.flags.encode_into(writer)?;
        if let Some(os_info) = &self.os_info {
            os_info.encode_into(writer)?;
        }
        Ok(())
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for SystemCapset {
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let flags = SystemCapsetFlags::decode_from(cursor)?;
        let os_info = if flags.os_info() {
            Some(NowSystemOsInfo::decode_from(cursor)?)
        } else {
            None
        };
        Ok(SystemCapset { flags, os_info })
    }
}

// unknown capset (not specified)

#[derive(Debug, Clone)]
pub struct UnknownCapset<'a> {
    // capset struct full size (including size bits and name)
    pub size: u16,
    pub name: NowString64,
    pub data: &'a [u8],
}

impl<'a> Encode for UnknownCapset<'a> {
    fn encoded_len(&self) -> usize {
        mem::size_of::<u16>() + self.name.encoded_len() + self.data.len()
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.size.encode_into(writer)?;
        self.name.encode_into(writer)?;
        for byte in self.data {
            byte.encode_into(writer)?;
        }
        Ok(())
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for UnknownCapset<'a> {
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let size = cursor.read_u16::<LittleEndian>()?;

        let name = NowString::decode_from(cursor)
            .chain(ProtoErrorKind::Decoding(__type_str!(UnknownCapset)))
            .or_desc("invalid capset name now string 64")?;

        let start_inclusive = cursor.position() as usize;
        let end_exclusive = start_inclusive + size as usize - mem::size_of_val(&size) - name.encoded_len();
        let data = &cursor.get_ref()[start_inclusive..end_exclusive];

        Ok(UnknownCapset { size, name, data })
    }
}

impl<'a> UnknownCapset<'a> {
    pub const REQUIRED_SIZE: usize = 4;
}

// NOW_CAPABILITIES_MSG

#[derive(Debug, Clone)]
pub enum NowCapset<'a> {
    Unknown(UnknownCapset<'a>),
    Transport(TransportCapset),
    Surface(SurfaceCapset),
    License(LicenseCapset),
    Access(AccessCapset),
    Update(UpdateCapset),
    Input(InputCapset),
    Mouse(MouseCapset),
    //TODO: Network(NetworkCapset),
    System(Box<SystemCapset>), // size difference is large...
}

impl NowCapset<'_> {
    pub fn name_as_str(&self) -> &str {
        match self {
            NowCapset::Unknown(msg) => msg.name.as_str(),
            NowCapset::Transport(_) => TransportCapset::NAME,
            NowCapset::Surface(_) => SurfaceCapset::NAME,
            NowCapset::License(_) => LicenseCapset::NAME,
            NowCapset::Access(_) => AccessCapset::NAME,
            NowCapset::Update(_) => UpdateCapset::NAME,
            NowCapset::Input(_) => InputCapset::NAME,
            NowCapset::Mouse(_) => MouseCapset::NAME,
            NowCapset::System(_) => SystemCapset::NAME,
        }
    }
}

macro_rules! encoded_len_capset_variant {
    ($capset:ident, $name:ident) => {
        $capset.encoded_len() + $name::NAME.len() + 2 + mem::size_of::<u16>()
    };
}

macro_rules! encode_capset_variant {
    ($capset:ident, $name:ident, $writer:ident) => {
        let name = unsafe { NowString64::from_str_unchecked($name::NAME) };

        let size = u16::try_from($capset.encoded_len() + name.encoded_len() + mem::size_of::<u16>())
            .map_err(ProtoError::from)
            .chain(ProtoErrorKind::Encoding(__type_str!(NowCapset)))
            .or_desc("capset data too large for the size field")?;

        size.encode_into($writer)?;
        name.encode_into($writer)?;
        $capset.encode_into($writer)?;
    };
}

impl<'a> Encode for NowCapset<'a> {
    fn encoded_len(&self) -> usize {
        match self {
            NowCapset::Unknown(capset) => capset.encoded_len(),
            NowCapset::Transport(capset) => encoded_len_capset_variant!(capset, TransportCapset),
            NowCapset::Surface(capset) => encoded_len_capset_variant!(capset, SurfaceCapset),
            NowCapset::License(capset) => encoded_len_capset_variant!(capset, LicenseCapset),
            NowCapset::Access(capset) => encoded_len_capset_variant!(capset, AccessCapset),
            NowCapset::Update(capset) => encoded_len_capset_variant!(capset, UpdateCapset),
            NowCapset::Input(capset) => encoded_len_capset_variant!(capset, InputCapset),
            NowCapset::Mouse(capset) => encoded_len_capset_variant!(capset, MouseCapset),
            NowCapset::System(capset) => encoded_len_capset_variant!(capset, SystemCapset),
        }
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            NowCapset::Unknown(capset) => capset.encode_into(writer)?,
            NowCapset::Transport(capset) => {
                encode_capset_variant! {capset, TransportCapset, writer}
            }
            NowCapset::Surface(capset) => {
                encode_capset_variant! {capset, SurfaceCapset, writer}
            }
            NowCapset::License(capset) => {
                encode_capset_variant! {capset, LicenseCapset, writer}
            }
            NowCapset::Access(capset) => {
                encode_capset_variant! {capset, AccessCapset, writer}
            }
            NowCapset::Update(capset) => {
                encode_capset_variant! {capset, UpdateCapset, writer}
            }
            NowCapset::Input(capset) => {
                encode_capset_variant! {capset, InputCapset, writer}
            }
            NowCapset::Mouse(capset) => {
                encode_capset_variant! {capset, MouseCapset, writer}
            }
            NowCapset::System(capset) => {
                encode_capset_variant! {capset, SystemCapset, writer}
            }
        }

        Ok(())
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for NowCapset<'a> {
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let size = u16::decode_from(cursor)?;
        let name = NowString64::decode_from(cursor)?;
        match name.as_str() {
            TransportCapset::NAME => Ok(Self::Transport(TransportCapset::decode_from(cursor)?)),
            SurfaceCapset::NAME => Ok(Self::Surface(SurfaceCapset::decode_from(cursor)?)),
            LicenseCapset::NAME => Ok(Self::License(LicenseCapset::decode_from(cursor)?)),
            AccessCapset::NAME => Ok(Self::Access(AccessCapset::decode_from(cursor)?)),
            UpdateCapset::NAME => Ok(Self::Update(UpdateCapset::decode_from(cursor)?)),
            InputCapset::NAME => Ok(Self::Input(InputCapset::decode_from(cursor)?)),
            MouseCapset::NAME => Ok(Self::Mouse(MouseCapset::decode_from(cursor)?)),
            SystemCapset::NAME => Ok(Self::System(Box::new(SystemCapset::decode_from(cursor)?))),
            _ => Ok(Self::Unknown(UnknownCapset {
                size,
                name,
                data: &cursor.get_ref()[cursor.position() as usize..],
            })),
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowCapabilitiesMsg<'a> {
    flags: u32,
    pub capabilities: Vec8<NowCapset<'a>>,
}

impl<'a> NowCapabilitiesMsg<'a> {
    pub fn new_with_capabilities(capabilities: Vec<NowCapset<'a>>) -> Self {
        Self {
            flags: 0,
            capabilities: Vec8(capabilities),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{
        EdgeRect, NowString128, NowString16, NowString32, NowString64, NowSurfaceDef, OsArch, OsType, VirtChannelsCtx,
    };
    use crate::packet::NowPacket;
    use core::str::FromStr;

    #[rustfmt::skip]
    const CAPABILITIES_PACKET: [u8; 388] = [
        // header
        0x80, 0x01, 0x05, 0x80,

        // flags
        0x00, 0x00, 0x00, 0x00,

        // count
        0x08,

        // transport
        0x14, 0x00, 0x0c, 0x4e, 0x6f, 0x77, 0x54, 0x72, 0x61, 0x6e,
        0x73, 0x70, 0x6f, 0x72, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00,

        // surface
        0x2b, 0x00, 0x0a, 0x4e, 0x6f, 0x77, 0x53, 0x75, 0x72, 0x66,
        0x61, 0x63, 0x65, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x03, 0x01, 0x10, 0x00, 0x09,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x03,

        // update
        0x2a, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x55, 0x70, 0x64, 0x61,
        0x74, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x08, 0x00, 0x02, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x00, 0x00,
        0x00, 0x00,

        // input
        0x35, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x49, 0x6e, 0x70, 0x75,
        0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x08, 0x10, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x12,
        0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00,
        0x00, 0x16, 0x00, 0x00, 0x00, 0x17, 0x00, 0x00, 0x00, 0x13,
        0x00, 0x00, 0x00,

        // mouse
        0x14, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x4d, 0x6f, 0x75, 0x73,
        0x65, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,

        // access
        0x2e, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x41, 0x63, 0x63, 0x65,
        0x73, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x06, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x00,
        0x03, 0x00, 0x01, 0x00, 0x04, 0x00, 0x01, 0x00, 0x05, 0x00,
        0x01, 0x00, 0x06, 0x00, 0x01, 0x00,

        // license
        0x12, 0x00, 0x0a, 0x4e, 0x6f, 0x77, 0x4c, 0x69, 0x63, 0x65,
        0x6e, 0x73, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00,

        // system
        0x89, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x53, 0x79, 0x73, 0x74,
        0x65, 0x6d, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02,
        0x00, 0x03, 0x02, 0x12, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x12, 0x55, 0x62, 0x75, 0x6e, 0x74, 0x75, 0x20, 0x31,
        0x38, 0x2e, 0x30, 0x34, 0x2e, 0x30, 0x20, 0x4c, 0x54, 0x53,
        0x00, 0x05, 0x4c, 0x69, 0x6e, 0x75, 0x78, 0x00, 0x06, 0x78,
        0x38, 0x36, 0x5f, 0x36, 0x34, 0x00, 0x10, 0x35, 0x2e, 0x30,
        0x2e, 0x30, 0x2d, 0x32, 0x39, 0x2d, 0x67, 0x65, 0x6e, 0x65,
        0x72, 0x69, 0x63, 0x00, 0x33, 0x23, 0x33, 0x31, 0x7e, 0x31,
        0x38, 0x2e, 0x30, 0x34, 0x2e, 0x31, 0x2d, 0x55, 0x62, 0x75,
        0x6e, 0x74, 0x75, 0x20, 0x53, 0x4d, 0x50, 0x20, 0x54, 0x68,
        0x75, 0x20, 0x53, 0x65, 0x70, 0x20, 0x31, 0x32, 0x20, 0x31,
        0x38, 0x3a, 0x32, 0x39, 0x3a, 0x32, 0x31, 0x20, 0x55, 0x54,
        0x43, 0x20, 0x32, 0x30, 0x31, 0x39, 0x00
    ];

    const CAPABILITIES_WINDOWS_ARCH_PACKET: [u8; 304] = [
        0x2c, 0x01, 0x05, 0x80, 0x00, 0x00, 0x00, 0x00, 0x08, 0x14, 0x00, 0x0c, 0x4e, 0x6f, 0x77, 0x54, 0x72, 0x61,
        0x6e, 0x73, 0x70, 0x6f, 0x72, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x00, 0x0a, 0x4e, 0x6f, 0x77, 0x53,
        0x75, 0x72, 0x66, 0x61, 0x63, 0x65, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x40, 0x06, 0x84,
        0x03, 0x01, 0x10, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x06, 0x84, 0x03,
        0x2a, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x08, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x49, 0x6e, 0x70, 0x75, 0x74, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x10, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x12,
        0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x17, 0x00, 0x00,
        0x00, 0x14, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x4d, 0x6f, 0x75, 0x73, 0x65, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x2e, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x41, 0x63, 0x63, 0x65, 0x73, 0x73, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x00, 0x03, 0x00, 0x01,
        0x00, 0x04, 0x00, 0x01, 0x00, 0x05, 0x00, 0x01, 0x00, 0x06, 0x00, 0x01, 0x00, 0x12, 0x00, 0x0a, 0x4e, 0x6f,
        0x77, 0x4c, 0x69, 0x63, 0x65, 0x6e, 0x73, 0x65, 0x00, 0x01, 0x00, 0x00, 0x00, 0x39, 0x00, 0x09, 0x4e, 0x6f,
        0x77, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6d, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x02,
        0x06, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x39, 0x32, 0x30, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn full_decode() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&CAPABILITIES_PACKET[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &VirtChannelsCtx::new()) {
            Ok(_) => {}
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode capabilities packet");
            }
        }
    }

    #[test]
    fn full_decode_windows() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&CAPABILITIES_WINDOWS_ARCH_PACKET[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &VirtChannelsCtx::new()) {
            Ok(_) => {}
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode capabilities packet");
            }
        }
    }

    #[test]
    fn full_encode() {
        let capabilities = vec![
            NowCapset::Transport(TransportCapset::default()),
            NowCapset::Surface(SurfaceCapset::new(
                SurfaceCapsetFlags::new_empty().set_list_req().set_select(),
                NowSurfaceListReqMsg::new_with_surfaces(
                    0,
                    1024,
                    768,
                    vec![NowSurfaceDef::new(
                        0,
                        EdgeRect {
                            left: 0,
                            top: 0,
                            right: 1024,
                            bottom: 768,
                        },
                    )],
                ),
            )),
            NowCapset::Update(UpdateCapset::new_with_supported_codecs(vec![
                NowCodecDef::new_with_flags(Codec::JPEG, 0x0000_0001),
                NowCodecDef::new(Codec::GFWX),
            ])),
            NowCapset::Input(InputCapset::new_with_actions(vec![
                NowInputActionDef::new_enabled(InputActionCode::ClipboardCut),
                NowInputActionDef::new_enabled(InputActionCode::ClipboardCopy),
                NowInputActionDef::new_enabled(InputActionCode::ClipboardPaste),
                NowInputActionDef::new_enabled(InputActionCode::ClipboardPasteSpecial),
                NowInputActionDef::new_enabled(InputActionCode::SelectAll),
                NowInputActionDef::new_enabled(InputActionCode::Undo),
                NowInputActionDef::new_enabled(InputActionCode::Redo),
                NowInputActionDef::new_enabled(InputActionCode::ClipboardCopySpecial),
            ])),
            NowCapset::Mouse(MouseCapset::new(
                MouseMode::Primary,
                MouseCapsetFlags::new_empty().set_large(),
            )),
            NowCapset::Access(AccessCapset::new_with_access_controls(vec![
                AccessControlDef::new_allowed(AccessControlCode::Viewing),
                AccessControlDef::new_allowed(AccessControlCode::Interact),
                AccessControlDef::new_allowed(AccessControlCode::Clipboard),
                AccessControlDef::new_allowed(AccessControlCode::FileTransfer),
                AccessControlDef::new_allowed(AccessControlCode::Exec),
                AccessControlDef::new_allowed(AccessControlCode::Chat),
            ])),
            NowCapset::License(LicenseCapset {
                flags: LicenseCapsetFlags::new_empty(),
            }),
            NowCapset::System(Box::new(SystemCapset::new_os_info({
                let mut info = NowSystemOsInfo::new(OsType::Linux, OsArch::X64, 18, 4, 0, NowString16::new_empty());
                info.set_kernel_infos(
                    NowString64::from_str("Ubuntu 18.04.0 LTS").unwrap(),
                    NowString16::from_str("Linux").unwrap(),
                    NowString16::from_str("x86_64").unwrap(),
                    NowString32::from_str("5.0.0-29-generic").unwrap(),
                    NowString128::from_str("#31~18.04.1-Ubuntu SMP Thu Sep 12 18:29:21 UTC 2019").unwrap(),
                );
                info
            }))),
        ];
        let packet = NowPacket::from_message(NowCapabilitiesMsg::new_with_capabilities(capabilities));
        assert_eq!(packet.encode().unwrap(), CAPABILITIES_PACKET.to_vec());
    }

    #[rustfmt::skip]
    const UPDATE_CAPSET: [u8; 42] = [
        // size
        0x2a, 0x00,
        // name
        0x09, 0x4e, 0x6f, 0x77, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x00,
        // flags
        0x00, 0x00, 0x00, 0x00,
        // quality mode
        0x00,
        // padding
        0x00,
        // codec id
        0x00, 0x00,
        // performance
        0x00, 0x00, 0x00, 0x00,
        // codecs count
        0x02,
        // codec 0
        0x08, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00,
        // codec 1
        0x08, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn decode_update_capset() {
        let capset = NowCapset::decode(&UPDATE_CAPSET).unwrap();
        if let NowCapset::Update(capset) = capset {
            assert_eq!(capset.flags, 0);
            assert_eq!(capset.quality_mode, QualityMode::Unspecified);
            assert_eq!(capset.padding, 0);
            assert_eq!(capset.codec_id, Codec::Unspecified);
            assert_eq!(capset.performance, 0);
            assert_eq!(capset.codecs.len(), 2);
            let codec = &capset.codecs[0];
            assert_eq!(codec.size, 8);
            assert_eq!(codec.id, Codec::JPEG);
            assert_eq!(codec.flags, 0x0000_0001);
        } else {
            panic!("expected an update capset got {:?}", capset);
        }
    }

    #[test]
    fn encode_update_capset() {
        let codecs = vec![
            NowCodecDef::new_with_flags(Codec::JPEG, 0x0000_0001),
            NowCodecDef::new(Codec::GFWX),
        ];
        let capset = NowCapset::Update(UpdateCapset::new_with_supported_codecs(codecs));
        assert_eq!(capset.encode().unwrap(), UPDATE_CAPSET.to_vec());
    }

    #[rustfmt::skip]
    const INPUT_CAPSET: [u8; 53] = [
        // size
        0x35, 0x00,
        // name
        0x08, 0x4e, 0x6f, 0x77, 0x49, 0x6e, 0x70, 0x75, 0x74, 0x00,
        // flags
        0x00, 0x00, 0x00, 0x00,
        // reserved
        0x00, 0x00, 0x00, 0x00,
        // action def count
        0x08,
        // def 0
        0x10, 0x00, 0x00, 0x00,
        // def 1
        0x11, 0x00, 0x00, 0x00,
        // def 2
        0x12, 0x00, 0x00, 0x00,
        // def 3
        0x14, 0x00, 0x00, 0x00,
        // def 4
        0x15, 0x00, 0x00, 0x00,
        // def 5
        0x16, 0x00, 0x00, 0x00,
        // def 6
        0x17, 0x00, 0x00, 0x00,
        // def 7
        0x13, 0x00, 0x00, 0x00
    ];

    #[test]
    fn decode_input_capset() {
        let capset = NowCapset::decode(&INPUT_CAPSET).unwrap();
        if let NowCapset::Input(capset) = capset {
            assert_eq!(capset.flags, 0);
            assert_eq!(capset.reserved, 0);
            assert_eq!(capset.actions.len(), 8);
            let action_def = &capset.actions[0];
            assert_eq!(action_def.code, InputActionCode::ClipboardCut);
            assert!(!action_def.flags.disabled());
        } else {
            panic!("expected an update capset got {:?}", capset);
        }
    }

    #[test]
    fn encode_input_capset() {
        let actions = vec![
            NowInputActionDef::new_enabled(InputActionCode::ClipboardCut),
            NowInputActionDef::new_enabled(InputActionCode::ClipboardCopy),
            NowInputActionDef::new_enabled(InputActionCode::ClipboardPaste),
            NowInputActionDef::new_enabled(InputActionCode::ClipboardPasteSpecial),
            NowInputActionDef::new_enabled(InputActionCode::SelectAll),
            NowInputActionDef::new_enabled(InputActionCode::Undo),
            NowInputActionDef::new_enabled(InputActionCode::Redo),
            NowInputActionDef::new_enabled(InputActionCode::ClipboardCopySpecial),
        ];
        let capset = NowCapset::Input(InputCapset::new_with_actions(actions));
        assert_eq!(capset.encode().unwrap(), INPUT_CAPSET.to_vec());
    }

    #[rustfmt::skip]
    const UNKNOWN_CAPSET: [u8; 25] = [
        // size (size bits + name + data)
        0x19, 0x00,
        // name
        17, 115, 111, 109, 101, 116, 104, 105, 110, 103,
        95, 117, 110, 107, 110, 111, 119, 110, 0,
        // data
        0x02, 0x29, 0x85, 0x12,
    ];

    #[test]
    fn decode_unknown_capset() {
        let capset = NowCapset::decode(&UNKNOWN_CAPSET).unwrap();
        if let NowCapset::Unknown(capset) = capset {
            assert_eq!(capset.size, 25);
            assert_eq!(capset.name.as_str(), "something_unknown");
            assert_eq!(capset.data, &[0x02, 0x29, 0x85, 0x12]);
        } else {
            panic!("expected an unknown capset got {:?}", capset);
        }
    }

    #[test]
    fn encode_unknown_capset() {
        let capset = NowCapset::Unknown(UnknownCapset {
            size: 25,
            name: NowString64::from_str("something_unknown").unwrap(),
            data: &[0x02, 0x29, 0x85, 0x12],
        });
        assert_eq!(capset.encode().unwrap(), UNKNOWN_CAPSET.to_vec(),)
    }

    const PACKET_WITHOUT_OS_INFO: [u8; 268] = [
        0x08, 0x01, 0x05, 0x80, 0x00, 0x00, 0x00, 0x00, 0x08, 0x14, 0x00, 0x0c, 0x4e, 0x6f, 0x77, 0x54, 0x72, 0x61,
        0x6e, 0x73, 0x70, 0x6f, 0x72, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x00, 0x0a, 0x4e, 0x6f, 0x77, 0x53,
        0x75, 0x72, 0x66, 0x61, 0x63, 0x65, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3d, 0x07, 0xc5,
        0x03, 0x01, 0x10, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3d, 0x07, 0xc5, 0x03,
        0x2a, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x08, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x35, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x49, 0x6e, 0x70, 0x75, 0x74, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x10, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x12,
        0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x17, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x14, 0x00, 0x08, 0x4e, 0x6f, 0x77, 0x4d, 0x6f, 0x75, 0x73, 0x65, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x2e, 0x00, 0x09, 0x4e, 0x6f, 0x77, 0x41, 0x63, 0x63, 0x65, 0x73,
        0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00, 0x01,
        0x00, 0x03, 0x00, 0x01, 0x00, 0x04, 0x00, 0x01, 0x00, 0x05, 0x00, 0x01, 0x00, 0x06, 0x00, 0x01, 0x00, 0x12,
        0x00, 0x0a, 0x4e, 0x6f, 0x77, 0x4c, 0x69, 0x63, 0x65, 0x6e, 0x73, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11,
        0x00, 0x09, 0x4e, 0x6f, 0x77, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn full_decode_packet_without_os_info() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&PACKET_WITHOUT_OS_INFO[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &VirtChannelsCtx::new()) {
            Ok(_) => {}
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode capabilities packet");
            }
        }
    }
}
