use crate::{
    error::*,
    serialization::{Decode, Encode},
};
use core::{convert::TryFrom, fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use std::io::{Cursor, Write};

// NSTATUS

#[derive(Debug, Clone)]
pub struct NowStatusBuilder<CodeType> {
    severity: SeverityLevel,
    status_type: StatusType,
    code: CodeType,
}

impl<CodeType> NowStatusBuilder<CodeType>
where
    CodeType: num::ToPrimitive,
{
    pub fn severity<V: Into<SeverityLevel>>(self, value: V) -> Self {
        Self {
            severity: value.into(),
            ..self
        }
    }

    pub fn status_type<V: Into<StatusType>>(self, value: V) -> Self {
        Self {
            status_type: value.into(),
            ..self
        }
    }

    pub fn build(self) -> NowStatus<CodeType> {
        let repr = ((self.severity as u32) << 30)
            + ((self.status_type as u32) << 16)
            + num::ToPrimitive::to_u32(&self.code).unwrap(); // should not panic.

        NowStatus {
            repr,
            severity: self.severity,
            status_type: self.status_type,
            code: self.code,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NowStatus<CodeType> {
    repr: u32,

    // cache
    severity: SeverityLevel,
    status_type: StatusType,
    code: CodeType,
}

impl<CodeType> Encode for NowStatus<CodeType> {
    fn encoded_len(&self) -> usize {
        std::mem::size_of::<u32>()
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.repr.encode_into(writer)
    }
}

impl<CodeType> Decode<'_> for NowStatus<CodeType>
where
    CodeType: num::FromPrimitive,
{
    fn decode_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let repr = u32::decode_from(cursor)?;
        Self::from_u32(repr)
    }
}

impl<CodeType> TryFrom<u32> for NowStatus<CodeType>
where
    CodeType: num::FromPrimitive,
{
    type Error = ProtoError;

    fn try_from(repr: u32) -> Result<Self> {
        Ok(NowStatus {
            repr,
            severity: num::FromPrimitive::from_u32(repr >> 30)
                .chain(ProtoErrorKind::Decoding(stringify!(NowStatus)))
                .or_desc("couldn't parse severity")?,
            status_type: num::FromPrimitive::from_u32((repr & 0x00FF_0000) >> 16)
                .chain(ProtoErrorKind::Decoding(stringify!(NowStatus)))
                .or_desc("couldn't parse status type")?,
            code: num::FromPrimitive::from_u32(repr & 0x0000_FFFF)
                .chain(ProtoErrorKind::Decoding(stringify!(NowStatus)))
                .or_desc("couldn't parse status code")?,
        })
    }
}

impl<CodeType> Into<u32> for NowStatus<CodeType> {
    fn into(self) -> u32 {
        self.repr
    }
}

impl<CodeType> PartialEq<u32> for NowStatus<CodeType> {
    fn eq(&self, other: &u32) -> bool {
        self.repr == *other
    }
}

impl<CodeType> Default for NowStatus<CodeType>
where
    CodeType: num::FromPrimitive + num::ToPrimitive,
{
    fn default() -> Self {
        // should not panic... Code 0 should be "Success" for any CodeType.
        NowStatus::builder(<CodeType as num::FromPrimitive>::from_u8(0).unwrap()).build()
    }
}

impl<CodeType> NowStatus<CodeType>
where
    CodeType: num::FromPrimitive,
{
    pub fn from_u32(repr: u32) -> Result<Self> {
        Self::try_from(repr)
    }
}

impl<CodeType> NowStatus<CodeType> {
    pub fn builder<V: Into<CodeType>>(code: V) -> NowStatusBuilder<CodeType> {
        NowStatusBuilder {
            severity: SeverityLevel::Info,
            status_type: StatusType::None,
            code: code.into(),
        }
    }
}

impl<CodeType> NowStatus<CodeType>
where
    CodeType: Copy,
{
    pub fn severity(&self) -> SeverityLevel {
        self.severity
    }

    pub fn status_type(&self) -> StatusType {
        self.status_type
    }

    pub fn code(&self) -> CodeType {
        self.code
    }

    pub fn as_u32(&self) -> u32 {
        self.repr
    }
}

impl<CodeType> fmt::Display for NowStatus<CodeType>
where
    CodeType: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.status_type, self.severity, self.code)
    }
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SeverityLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
    Fatal = 3,
}

impl fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum StatusType {
    None = 0,
    Disconnect = 1,
    Connect = 2,
    Security = 3,
    Handshake = 21,
    Negotiate = 22,
    Auth = 23,
    Associate = 24,
    Capabilities = 25,
    Channel = 26,
    Clipboard = 0x81,
    FileTransfer = 0x82,
    Exec = 0x83,
}

impl fmt::Display for StatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Common

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
enum StatusCode {
    Success = 0x0000,
    Failure = 0xFFFF,
}

// NSTATUS_DISCONNECT_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum DisconnectStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    ByLocalUser = 1,
    ByRemoteUser = 2,
    ByLocalSystem = 3,
    ByRemoteSystem = 4,
    SystemShutdown = 5,
    SystemReboot = 6,
    LocalLogoff = 7,
    RemoteLogoff = 8,
    ByOtherConnection = 9,
    LogonTimeout = 10,
    LogonCancelled = 11,
    IdleTimeout = 12,
    AlreadyActive = 13,
    LicenseRequired = 14,
}

impl fmt::Display for DisconnectStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "disconnected with success"),
            Self::Failure => write!(f, "disconnection with unknown failure"),
            Self::ByLocalUser => write!(f, "disconnected by local user."),
            Self::ByRemoteUser => write!(f, "disconnected by remote user."),
            Self::ByLocalSystem => write!(f, "disconnected by local system."),
            Self::ByRemoteSystem => write!(f, "disconnected by remote system."),
            Self::SystemShutdown => write!(f, "disconnected because of system shutdown."),
            Self::SystemReboot => write!(f, "disconnected because of system reboot."),
            Self::LocalLogoff => write!(f, "disconnected because of local logoff."),
            Self::RemoteLogoff => write!(f, "disconnected because of remote logoff."),
            Self::ByOtherConnection => write!(f, "disconnected by another connexion."),
            Self::LogonTimeout => write!(f, "disconnected because of logon timeout."),
            Self::LogonCancelled => write!(f, "disconnected because the logon was canceled."),
            Self::IdleTimeout => write!(f, "disconnected because of idle timeout."),
            Self::AlreadyActive => write!(f, "disconnected because another connection is already active."),
            Self::LicenseRequired => write!(f, "disconnected because a license is required."),
        }
    }
}

// NSTATUS_CONNECT_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum ConnectStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    Unresolved = 1,
    Unreachable = 2,
    Refused = 3,
    Loopback = 4,
    Concurrent = 5,
    Unauthorized = 6,
}

impl fmt::Display for ConnectStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "connection succeeded"),
            Self::Failure => write!(f, "connection failed"),
            Self::Unresolved => write!(f, "connection unresolved"),
            Self::Unreachable => write!(f, "sharer unreachable"),
            Self::Refused => write!(f, "sharer refused connection"),
            Self::Loopback => write!(f, "connection loopback"),
            Self::Concurrent => write!(f, "concurrent connection"),
            Self::Unauthorized => write!(f, "unauthorized connection"),
        }
    }
}

// NSTATUS_SECURITY_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum SecurityStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    TLSHandshake = 1,
    TLSClientCert = 2,
    TLSServerCert = 3,
}

impl fmt::Display for SecurityStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "unknown failure"),
            Self::TLSHandshake => write!(f, "TLS failed"),
            Self::TLSClientCert => write!(f, "bad client TLS certificate"),
            Self::TLSServerCert => write!(f, "bad server TLS certificate"),
        }
    }
}

// NSTATUS_HANDSHAKE_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum HandshakeStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    Incompatible = 1,
}

impl fmt::Display for HandshakeStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "handshake succeeded"),
            Self::Failure => write!(f, "handshaked failed"),
            Self::Incompatible => write!(f, "version is incompatible"),
        }
    }
}

// NSTATUS_NEGOTIATE_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum NegotiateStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for NegotiateStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "negotiation succeeded"),
            Self::Failure => write!(f, "negotiation failed"),
        }
    }
}

// NSTATUS_AUTH_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum AuthStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    Timeout = 1,
    Cancelled = 2,
    AccountDisabled = 3,
    AccountExpired = 4,
    AccountRestriction = 5,
    InvalidLogonHours = 6,
    InvalidWorkstation = 7,
    PasswordExpired = 8,
    PasswordMustChange = 9,
}

impl fmt::Display for AuthStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "authentication succeeded"),
            Self::Failure => write!(f, "authentication failed"),
            Self::Timeout => write!(f, "timed out"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::AccountDisabled => write!(f, "account disabled"),
            Self::AccountExpired => write!(f, "account expired"),
            Self::AccountRestriction => write!(f, "account restricted"),
            Self::InvalidLogonHours => write!(f, "invalid logon hours"),
            Self::InvalidWorkstation => write!(f, "invalid workstation"),
            Self::PasswordExpired => write!(f, "password expired"),
            Self::PasswordMustChange => write!(f, "password must change"),
        }
    }
}

// NSTATUS_ASSOCIATE_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum AssociateStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for AssociateStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "association succeeded"),
            Self::Failure => write!(f, "association failed"),
        }
    }
}

// NSTATUS_CAPABILITIES_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum CapabilitiesStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for CapabilitiesStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failure => write!(f, "capabilities negotiation failed"),
            Self::Success => write!(f, "capabilities negotiation succeeded"),
        }
    }
}

// NSTATUS_CHANNEL_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum ChannelStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for ChannelStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
        }
    }
}

// NSTATUS_CLIPBOARD_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum ClipboardStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for ClipboardStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
        }
    }
}

// NSTATUS_FILE_TRANSFER_TYPE

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum FileTransferStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
}

impl fmt::Display for FileTransferStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTransferStatusCode::Success => write!(f, "success"),
            FileTransferStatusCode::Failure => write!(f, "failure"),
        }
    }
}

// NSTATUS_EXEC_TYPE (Remote Execution)

#[derive(Encode, Decode, FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum ExecStatusCode {
    Success = StatusCode::Success as u16,
    Failure = StatusCode::Failure as u16,
    FileNotFound = 1,
    InvalidExecutable = 2,
    AccessDenied = 3,
}

impl fmt::Display for ExecStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecStatusCode::Success => write!(f, "execution success"),
            ExecStatusCode::Failure => write!(f, "execution failed"),
            ExecStatusCode::FileNotFound => write!(f, "file not found"),
            ExecStatusCode::InvalidExecutable => write!(f, "invalid executable"),
            ExecStatusCode::AccessDenied => write!(f, "access denied"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num;

    #[test]
    fn integer_conversion() {
        let status = num::FromPrimitive::from_u8(3);
        match status {
            Some(SeverityLevel::Fatal) => { /* success */ }
            _ => panic!("wrong status value"),
        }
    }

    #[test]
    fn parse_from_u32() {
        let nstatus = NowStatus::<AuthStatusCode>::try_from(0x8017_ffff).unwrap();
        assert_eq!(nstatus.severity(), SeverityLevel::Error);
        assert_eq!(nstatus.code(), AuthStatusCode::Failure);
        assert_eq!(nstatus.status_type(), StatusType::Auth);
    }

    #[test]
    fn repr_building() {
        let nstatus = NowStatus::<AuthStatusCode>::builder(AuthStatusCode::Failure)
            .severity(SeverityLevel::Error)
            .status_type(StatusType::Auth)
            .build();
        assert_eq!(nstatus.as_u32(), 0x8017_ffff);
    }
}
