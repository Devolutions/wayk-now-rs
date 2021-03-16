use crate::error::*;
use crate::message::{NowString128, NowString16, NowString256, NowString32, NowString64};
use crate::serialization::{Decode, Encode};
use std::io::{Cursor, Seek, SeekFrom, Write};

// NOW_SYSTEM_INFO

__flags_struct! {
    WindowsProductFlags: u16 => {
        client = CLIENT = 0x0001,
        server = SERVER = 0x0002,
    }
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct OsInfoExtraWindows {
    pub extra_flags: u16,
    pub product_flags: WindowsProductFlags,
    // Windows Update Build Revision
    pub ubr: u32,
    pub release_id: u32,
    pub service_pack_major: u16,
    pub service_pack_minor: u16,
    pub edition_id: NowString32,
    pub product_name: NowString64,
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct OsInfoExtraMac {
    pub extra_flags: u16,
    reserved: u16,
}

impl OsInfoExtraMac {
    pub fn new_with_flags(extra_flags: u16) -> Self {
        Self {
            extra_flags,
            reserved: 0,
        }
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct OsInfoExtraLinux {
    pub extra_flags: u16,
    reserved: u16,
}

impl OsInfoExtraLinux {
    pub fn new_with_flags(extra_flags: u16) -> Self {
        Self {
            extra_flags,
            reserved: 0,
        }
    }
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct OsInfoExtraIOS {
    pub extra_flags: u16,
    reserved: u16,
}

impl OsInfoExtraIOS {
    pub fn new_with_flags(extra_flags: u16) -> Self {
        Self {
            extra_flags,
            reserved: 0,
        }
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct OsInfoExtraAndroid {
    pub extra_flags: u16,
    reserved: u16,
}

impl OsInfoExtraAndroid {
    pub fn new_with_flags(extra_flags: u16) -> Self {
        Self {
            extra_flags,
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Encode)]
#[meta_enum = "None"]
pub enum OsInfoExtra {
    Windows(OsInfoExtraWindows),
    Mac(OsInfoExtraMac),
    Linux(OsInfoExtraLinux),
    IOS(OsInfoExtraIOS),
    Android(OsInfoExtraAndroid),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SystemInfoType {
    #[value = 0x0001]
    Os,
    #[fallback]
    Other(u16),
}

impl SystemInfoType {
    pub fn size() -> usize {
        2
    }
}

__flags_struct! {
    SystemOsInfoFlags: u16 => {
        extra = EXTRA = 0x0001,
        kernel = KERNEL = 0x0002,
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum OsType {
    #[value = 0x01]
    Windows,
    #[value = 0x02]
    Mac,
    #[value = 0x03]
    Linux,
    #[value = 0x04]
    IOS,
    #[value = 0x5]
    Android,
    #[fallback]
    Other(u8),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum OsArch {
    #[value = 0x01]
    X86,
    #[value = 0x02]
    X64,
    #[value = 0x03]
    ARM,
    #[value = 0x04]
    ARM64,
    #[fallback]
    Other(u8),
}

#[derive(Debug, Clone)]
pub struct NowSystemOsInfo {
    subtype: SystemInfoType,
    pub flags: SystemOsInfoFlags,

    pub os_type: OsType,
    pub os_arch: OsArch,
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub os_build: NowString16,
    pub os_name: NowString64,
    pub kernel_name: NowString16,
    pub kernel_arch: NowString16,
    pub kernel_release: NowString32,
    pub kernel_version: NowString128,

    pub extra: Option<OsInfoExtra>,
}

impl Encode for NowSystemOsInfo {
    fn encoded_len(&self) -> usize {
        self.subtype.encoded_len()
            + self.flags.encoded_len()
            + self.os_type.encoded_len()
            + self.os_arch.encoded_len()
            + self.version_major.encoded_len()
            + self.version_minor.encoded_len()
            + self.version_patch.encoded_len()
            + self.os_build.encoded_len()
            + self.os_name.encoded_len()
            + self.kernel_name.encoded_len()
            + self.kernel_arch.encoded_len()
            + self.kernel_release.encoded_len()
            + self.kernel_version.encoded_len()
            + if let Some(extra) = &self.extra {
                extra.encoded_len()
            } else {
                0
            }
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.subtype
            .encode_into(writer)
            .or_desc("couldn't encode os info subtype")?;
        self.flags
            .encode_into(writer)
            .or_desc("couldn't encode os info flags")?;
        self.os_type
            .encode_into(writer)
            .or_desc("couldn't encode os info os type")?;
        self.os_arch
            .encode_into(writer)
            .or_desc("couldn't encode os info os arch")?;
        self.version_major
            .encode_into(writer)
            .or_desc("couldn't encode os info version major")?;
        self.version_minor
            .encode_into(writer)
            .or_desc("couldn't encode os info version minor")?;
        self.version_patch
            .encode_into(writer)
            .or_desc("couldn't encode os info version patch")?;
        self.os_build
            .encode_into(writer)
            .or_desc("couldn't encode os info os build")?;
        self.os_name
            .encode_into(writer)
            .or_desc("couldn't encode os info os name")?;
        self.kernel_name
            .encode_into(writer)
            .or_desc("couldn't encode os info kernel name")?;
        self.kernel_arch
            .encode_into(writer)
            .or_desc("couldn't encode os info kernel arch")?;
        self.kernel_release
            .encode_into(writer)
            .or_desc("couldn't encode os info kernel release")?;
        self.kernel_version
            .encode_into(writer)
            .or_desc("couldn't encode os info kernel version")?;

        if let Some(extra) = &self.extra {
            extra.encode_into(writer).or_desc("couldn't encode os info extra")?;
        }

        Ok(())
    }
}

impl Decode<'_> for NowSystemOsInfo {
    fn decode_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor.seek(SeekFrom::Current(SystemInfoType::size() as i64))?;

        let flags = SystemOsInfoFlags::decode_from(cursor).or_desc("couldn't decode os info flags")?;
        let os_type = OsType::decode_from(cursor).or_desc("couldn't decode os type")?;
        let os_arch = OsArch::decode_from(cursor).or_desc("couldn't decode os arch")?;
        let version_major = u16::decode_from(cursor).or_desc("couldn't decode version major")?;
        let version_minor = u16::decode_from(cursor).or_desc("couldn't decode version minor")?;
        let version_patch = u16::decode_from(cursor).or_desc("couldn't decode version patch")?;
        let os_build = NowString16::decode_from(cursor).or_desc("couldn't decode os build")?;
        let os_name = NowString64::decode_from(cursor).or_desc("couldn't decode os name")?;
        let kernel_name;
        let kernel_arch;
        let kernel_release;
        let kernel_version;

        {
            if flags.kernel() {
                kernel_name = NowString16::decode_from(cursor).or_desc("couldn't decode kernel name")?;
                kernel_arch = NowString16::decode_from(cursor).or_desc("couldn't decode kernel arch")?;
                kernel_release = NowString32::decode_from(cursor).or_desc("couldn't decode kernel release")?;
                kernel_version = NowString128::decode_from(cursor).or_desc("couldn't decode kernel version")?;
            } else {
                kernel_name = NowString16::new_empty();
                kernel_arch = NowString16::new_empty();
                kernel_release = NowString32::new_empty();
                kernel_version = NowString128::new_empty();
            }
        }

        let extra = if flags.extra() {
            match os_type {
                OsType::Windows => Some(OsInfoExtra::Windows(OsInfoExtraWindows::decode_from(cursor)?)),
                OsType::Mac => Some(OsInfoExtra::Mac(OsInfoExtraMac::decode_from(cursor)?)),
                OsType::Linux => Some(OsInfoExtra::Linux(OsInfoExtraLinux::decode_from(cursor)?)),
                OsType::IOS => Some(OsInfoExtra::IOS(OsInfoExtraIOS::decode_from(cursor)?)),
                OsType::Android => Some(OsInfoExtra::Android(OsInfoExtraAndroid::decode_from(cursor)?)),
                OsType::Other(_) => None,
            }
        } else {
            None
        };

        Ok(Self {
            subtype: SystemInfoType::Os,
            flags,
            os_type,
            os_arch,
            version_major,
            version_minor,
            version_patch,
            os_build,
            os_name,
            kernel_name,
            kernel_arch,
            kernel_release,
            kernel_version,
            extra,
        })
    }
}

impl NowSystemOsInfo {
    const SUBTYPE: SystemInfoType = SystemInfoType::Os;

    pub fn new(
        os_type: OsType,
        os_arch: OsArch,
        version_major: u16,
        version_minor: u16,
        version_patch: u16,
        os_build: NowString16,
    ) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: SystemOsInfoFlags::new_empty(),
            os_type,
            os_arch,
            version_major,
            version_minor,
            version_patch,
            os_build,
            os_name: NowString64::new_empty(),
            kernel_name: NowString16::new_empty(),
            kernel_arch: NowString16::new_empty(),
            kernel_release: NowString32::new_empty(),
            kernel_version: NowString128::new_empty(),
            extra: None,
        }
    }

    pub fn set_kernel_infos(
        &mut self,
        os_name: NowString64,
        kernel_name: NowString16,
        kernel_arch: NowString16,
        kernel_release: NowString32,
        kernel_version: NowString128,
    ) {
        self.flags.set_kernel();
        self.os_name = os_name;
        self.kernel_name = kernel_name;
        self.kernel_arch = kernel_arch;
        self.kernel_release = kernel_release;
        self.kernel_version = kernel_version;
    }

    pub fn set_extra_infos(&mut self, extra: OsInfoExtra) {
        self.flags.set_extra();
        self.extra = Some(extra);
    }
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "SystemInfoType"]
pub enum NowSystemInfo {
    Os(NowSystemOsInfo),
}

// NOW_SYSTEM_MSG

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SystemMessageType {
    #[value = 0x01]
    InfoReq,
    #[value = 0x02]
    InfoRsp,
    #[value = 0x03]
    Shutdown,
    #[fallback]
    Other(u8),
}

#[derive(Debug, Decode, Encode, Clone)]
pub struct NowSystemInfoReqMsg {
    subtype: SystemMessageType,
    flags: u8,

    pub info_type: SystemInfoType,
}

impl NowSystemInfoReqMsg {
    pub const SUBTYPE: SystemMessageType = SystemMessageType::InfoReq;

    pub fn new(info_type: SystemInfoType) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            info_type,
        }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowSystemInfoRspMsg {
    subtype: SystemMessageType,
    flags: u8,

    pub info_data: NowSystemInfo,
}

impl NowSystemInfoRspMsg {
    pub const SUBTYPE: SystemMessageType = SystemMessageType::InfoRsp;

    pub fn new(info_data: NowSystemInfo) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            info_data,
        }
    }
}

__flags_struct! {
    ShutdownFlags: u8 => {
        force = FORCE = 0x01,
        reboot = REBOOT = 0x02,
    }
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct NowSystemShutdownMsg {
    subtype: SystemMessageType,
    pub flags: ShutdownFlags,

    reserved: u16,
    pub timeout: u32,
    reason: u32,
    pub message: NowString256,
}

impl NowSystemShutdownMsg {
    pub const SUBTYPE: SystemMessageType = SystemMessageType::Shutdown;

    pub fn new(flags: ShutdownFlags, timeout: u32, message: NowString256) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            reserved: 0,
            timeout,
            reason: 0,
            message,
        }
    }
}

#[derive(Debug, Clone, Decode, Encode)]
#[meta_enum = "SystemMessageType"]
pub enum NowSystemMsg {
    InfoReq(NowSystemInfoReqMsg),
    // size difference is large...
    InfoRsp(Box<NowSystemInfoRspMsg>),
    Shutdown(NowSystemShutdownMsg),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};
    use std::str::FromStr;

    #[rustfmt::skip]
    const SYSTEM_OS_INFO: [u8; 120] = [
        0x01, 0x00, // subtype
        0x02, 0x00, // flags

        0x03, // os type
        0x02, // os arch
        0x12, 0x00, // version major
        0x04, 0x00, // version minor
        0x00, 0x00, // version patch
        // os build
        0x00, 0x00,
        // os name
        0x12, 0x55, 0x62, 0x75, 0x6e, 0x74, 0x75, 0x20, 0x31, 0x38,
        0x2e, 0x30, 0x34, 0x2e, 0x30, 0x20, 0x4c, 0x54, 0x53, 0x00,
        // kernel name
        0x05, 0x4c, 0x69, 0x6e, 0x75, 0x78, 0x00,
        // kernel arch
        0x06, 0x78, 0x38, 0x36, 0x5f, 0x36, 0x34, 0x00,
        // kernel release
        0x10, 0x35, 0x2e, 0x30, 0x2e, 0x30, 0x2d, 0x32, 0x39, 0x2d,
        0x67, 0x65, 0x6e, 0x65, 0x72, 0x69, 0x63, 0x00,
        // kernel version
        0x33, 0x23, 0x33, 0x31, 0x7e, 0x31, 0x38, 0x2e, 0x30, 0x34,
        0x2e, 0x31, 0x2d, 0x55, 0x62, 0x75, 0x6e, 0x74, 0x75, 0x20,
        0x53, 0x4d, 0x50, 0x20, 0x54, 0x68, 0x75, 0x20, 0x53, 0x65,
        0x70, 0x20, 0x31, 0x32, 0x20, 0x31, 0x38, 0x3a, 0x32, 0x39,
        0x3a, 0x32, 0x31, 0x20, 0x55, 0x54, 0x43, 0x20, 0x32, 0x30,
        0x31, 0x39, 0x00
    ];

    #[test]
    fn decoding_info_data() {
        let info = NowSystemOsInfo::decode(&SYSTEM_OS_INFO).unwrap();
        assert_eq!(info.subtype, SystemInfoType::Os);
        assert!(info.flags.kernel());
        assert_eq!(info.os_type, OsType::Linux);
        assert_eq!(info.os_arch, OsArch::X64);
        assert_eq!(info.version_major, 18);
        assert_eq!(info.version_minor, 4);
        assert_eq!(info.version_patch, 0);
        assert_eq!(info.os_build, "");
        assert_eq!(info.os_name, "Ubuntu 18.04.0 LTS");
        assert_eq!(info.kernel_name, "Linux");
        assert_eq!(info.kernel_arch, "x86_64");
        assert_eq!(info.kernel_release, "5.0.0-29-generic");
        assert_eq!(
            info.kernel_version,
            "#31~18.04.1-Ubuntu SMP Thu Sep 12 18:29:21 UTC 2019"
        );
    }

    #[test]
    fn encoding_info_data() {
        let mut info = NowSystemOsInfo::new(OsType::Linux, OsArch::X64, 18, 4, 0, NowString16::new_empty());
        info.set_kernel_infos(
            NowString64::from_str("Ubuntu 18.04.0 LTS").unwrap(),
            NowString16::from_str("Linux").unwrap(),
            NowString16::from_str("x86_64").unwrap(),
            NowString32::from_str("5.0.0-29-generic").unwrap(),
            NowString128::from_str("#31~18.04.1-Ubuntu SMP Thu Sep 12 18:29:21 UTC 2019").unwrap(),
        );
        assert_eq!(info.encode().unwrap(), SYSTEM_OS_INFO.to_vec());
    }

    #[rustfmt::skip]
    const WINDOWS_SYSTEM_INFO: [u8; 40] = [
        0x01, 0x00, // u16 type
        0x01, 0x00, // u16 flags
        0x01, // u8 ostype
        0x02, // u8 osarch
        0x06, 0x00, // u16 major
        0x02, 0x00, // u16 minor
        0x00, 0x00, // u16 patch
        0x04, 0x39, 0x32, 0x30, 0x30, 0x00, // string16 os build string16
        0x00, 0x00, // u16 extra flags
        0x00, 0x00,// string64 osname
        0x01, 0x00, 0x00, 0x00,// u16 product flags
        0x00, 0x00, 0x00, 0x00, // u32 ubr
        0x00, 0x00, // u32 releasid
        0x00, 0x00, // u16 service pack major
        0x00, 0x00, // u16 service pack minor
        0x00, 0x00, // string32 edition id
        0x00, 0x00  // string64 product name
    ];

    #[test]
    fn decode_windows_info_packet() {
        NowSystemOsInfo::decode(&WINDOWS_SYSTEM_INFO).unwrap();
    }

    // TODO: info req message
}
