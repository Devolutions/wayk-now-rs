use crate::error::*;
use crate::io::{Cursor, NoStdWrite};
use crate::serialization::{Decode, Encode};
use core::convert::TryFrom;
use core::fmt;

// NSTATUS

#[derive(Debug, Clone)]
pub struct NowStatusBuilder<CodeType: From<u16> + Into<u16> + Copy> {
    severity: SeverityLevel,
    status_type: StatusType,
    code: CodeType,
}

impl<CodeType> NowStatusBuilder<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
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
        let repr = (u32::from(u8::from(self.severity)) << 30)
            + (u32::from(u8::from(self.status_type)) << 16)
            + u32::from(self.code.into());

        NowStatus {
            repr,
            severity: self.severity,
            status_type: self.status_type,
            code: self.code,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NowStatus<CodeType: From<u16> + Into<u16> + Copy> {
    repr: u32,

    // cache
    severity: SeverityLevel,
    status_type: StatusType,
    code: CodeType,
}

impl<CodeType> Encode for NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Known(core::mem::size_of::<u32>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<u32>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()> {
        self.repr.encode_into(writer)
    }
}

impl<CodeType> Decode<'_> for NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self> {
        let repr = u32::decode_from(cursor)?;
        Self::from_u32(repr)
    }
}

impl<CodeType> TryFrom<u32> for NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    type Error = ProtoError;

    fn try_from(repr: u32) -> Result<Self> {
        Ok(NowStatus {
            repr,
            severity: SeverityLevel::from(u8::try_from(repr >> 30)?),
            status_type: StatusType::from(u8::try_from((repr & 0x00FF_0000) >> 16)?),
            code: CodeType::from(u16::try_from(repr & 0x0000_FFFF)?),
        })
    }
}

impl<CodeType> From<NowStatus<CodeType>> for u32
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    fn from(v: NowStatus<CodeType>) -> Self {
        v.repr
    }
}

impl<CodeType> PartialEq<u32> for NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    fn eq(&self, other: &u32) -> bool {
        self.repr == *other
    }
}

impl<CodeType> Default for NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    fn default() -> Self {
        NowStatus::builder(0).build()
    }
}

impl<CodeType> NowStatus<CodeType>
where
    CodeType: From<u16> + Into<u16> + Copy,
{
    pub fn from_u32(repr: u32) -> Result<Self> {
        Self::try_from(repr)
    }

    pub fn builder<V: Into<CodeType>>(code: V) -> NowStatusBuilder<CodeType> {
        NowStatusBuilder {
            severity: SeverityLevel::Info,
            status_type: StatusType::None,
            code: code.into(),
        }
    }

    pub fn severity(&self) -> SeverityLevel {
        self.severity
    }

    pub fn status_type(&self) -> StatusType {
        self.status_type
    }

    pub fn as_u32(&self) -> u32 {
        self.repr
    }

    pub fn code(&self) -> CodeType {
        self.code
    }
}

impl<CodeType> fmt::Display for NowStatus<CodeType>
where
    CodeType: fmt::Display + From<u16> + Into<u16> + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.status_type, self.severity, self.code)
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SeverityLevel {
    #[value = 0x00]
    Info,
    #[value = 0x01]
    Warn,
    #[value = 0x02]
    Error,
    #[value = 0x03]
    Fatal,
    #[fallback]
    Other(u8),
}

impl fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum StatusType {
    #[value = 0x00]
    None,
    #[value = 0x01]
    Disconnect,
    #[value = 0x02]
    Connect,
    #[value = 0x03]
    Security,
    #[value = 0x15]
    Handshake,
    #[value = 0x16]
    Negotiate,
    #[value = 0x17]
    Auth,
    #[value = 0x18]
    Associate,
    #[value = 0x19]
    Capabilities,
    #[value = 0x1A]
    Channel,
    #[value = 0x81]
    Clipboard,
    #[value = 0x82]
    FileTransfer,
    #[value = 0x83]
    Exec,
    #[fallback]
    Other(u8),
}

impl fmt::Display for StatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// NSTATUS_DISCONNECT_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum DisconnectStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[value = 1]
    ByLocalUser,
    #[value = 2]
    ByRemoteUser,
    #[value = 3]
    ByLocalSystem,
    #[value = 4]
    ByRemoteSystem,
    #[value = 5]
    SystemShutdown,
    #[value = 6]
    SystemReboot,
    #[value = 7]
    LocalLogoff,
    #[value = 8]
    RemoteLogoff,
    #[value = 9]
    ByOtherConnection,
    #[value = 10]
    LogonTimeout,
    #[value = 11]
    LogonCancelled,
    #[value = 12]
    IdleTimeout,
    #[value = 13]
    AlreadyActive,
    #[value = 14]
    LicenseRequired,
    #[fallback]
    Other(u16),
}

impl fmt::Display for DisconnectStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "disconnected with success"),
            Self::Failure => write!(f, "disconnection with unknown failure"),
            Self::ByLocalUser => write!(f, "disconnected by local user"),
            Self::ByRemoteUser => write!(f, "disconnected by remote user"),
            Self::ByLocalSystem => write!(f, "disconnected by local system"),
            Self::ByRemoteSystem => write!(f, "disconnected by remote system"),
            Self::SystemShutdown => write!(f, "disconnected because of system shutdown"),
            Self::SystemReboot => write!(f, "disconnected because of system reboot"),
            Self::LocalLogoff => write!(f, "disconnected because of local logoff"),
            Self::RemoteLogoff => write!(f, "disconnected because of remote logoff"),
            Self::ByOtherConnection => write!(f, "disconnected by another connexion"),
            Self::LogonTimeout => write!(f, "disconnected because of logon timeout"),
            Self::LogonCancelled => write!(f, "disconnected because the logon was canceled"),
            Self::IdleTimeout => write!(f, "disconnected because of idle timeout"),
            Self::AlreadyActive => write!(f, "disconnected because another connection is already active"),
            Self::LicenseRequired => write!(f, "disconnected because a license is required"),
            Self::Other(v) => write!(f, "disconnect status code {}", v),
        }
    }
}

// NSTATUS_CONNECT_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ConnectStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[value = 1]
    Unresolved,
    #[value = 2]
    Unreachable,
    #[value = 3]
    Refused,
    #[value = 4]
    Loopback,
    #[value = 5]
    Concurrent,
    #[value = 6]
    Unauthorized,
    #[fallback]
    Other(u16),
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
            Self::Other(v) => write!(f, "connect status code {}", v),
        }
    }
}

// NSTATUS_SECURITY_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SecurityStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[value = 1]
    TLSHandshake,
    #[value = 2]
    TLSClientCert,
    #[value = 3]
    TLSServerCert,
    #[fallback]
    Other(u16),
}

impl fmt::Display for SecurityStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "unknown failure"),
            Self::TLSHandshake => write!(f, "TLS failed"),
            Self::TLSClientCert => write!(f, "bad client TLS certificate"),
            Self::TLSServerCert => write!(f, "bad server TLS certificate"),
            Self::Other(v) => write!(f, "security status code {}", v),
        }
    }
}

// NSTATUS_HANDSHAKE_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum HandshakeStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[value = 1]
    Incompatible,
    #[fallback]
    Other(u16),
}

impl fmt::Display for HandshakeStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "handshake succeeded"),
            Self::Failure => write!(f, "handshaked failed"),
            Self::Incompatible => write!(f, "version is incompatible"),
            Self::Other(v) => write!(f, "handshake status code {}", v),
        }
    }
}

// NSTATUS_NEGOTIATE_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum NegotiateStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for NegotiateStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "negotiation succeeded"),
            Self::Failure => write!(f, "negotiation failed"),
            Self::Other(v) => write!(f, "negotiation status code {}", v),
        }
    }
}

// NSTATUS_AUTH_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum AuthStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[value = 1]
    Timeout,
    #[value = 2]
    Cancelled,
    #[value = 3]
    AccountDisabled,
    #[value = 4]
    AccountExpired,
    #[value = 5]
    AccountRestriction,
    #[value = 6]
    InvalidLogonHours,
    #[value = 7]
    InvalidWorkstation,
    #[value = 8]
    PasswordExpired,
    #[value = 9]
    PasswordMustChange,
    #[fallback]
    Other(u16),
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
            Self::Other(v) => write!(f, "auth status code {}", v),
        }
    }
}

// NSTATUS_ASSOCIATE_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum AssociateStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for AssociateStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "association succeeded"),
            Self::Failure => write!(f, "association failed"),
            Self::Other(v) => write!(f, "association status code {}", v),
        }
    }
}

// NSTATUS_CAPABILITIES_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum CapabilitiesStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for CapabilitiesStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failure => write!(f, "capabilities negotiation failed"),
            Self::Success => write!(f, "capabilities negotiation succeeded"),
            Self::Other(v) => write!(f, "capabilities negotiation status code {}", v),
        }
    }
}

// NSTATUS_CHANNEL_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ChannelStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for ChannelStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "channel success"),
            Self::Failure => write!(f, "channel failure"),
            Self::Other(v) => write!(f, "channel status code {}", v),
        }
    }
}

// NSTATUS_CLIPBOARD_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ClipboardStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for ClipboardStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "clipboard success"),
            Self::Failure => write!(f, "clipboard failure"),
            Self::Other(v) => write!(f, "clipboard status code {}", v),
        }
    }
}

// NSTATUS_FILE_TRANSFER_TYPE

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum FileTransferStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for FileTransferStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileTransferStatusCode::Success => write!(f, "file transfer success"),
            FileTransferStatusCode::Failure => write!(f, "file transfer failure"),
            FileTransferStatusCode::Other(v) => write!(f, "file transfer status code {}", v),
        }
    }
}

// NSTATUS_EXEC_TYPE (Remote Execution)

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ExecStatusCode {
    #[value = 0x0000]
    Success,
    #[value = 0x0001]
    FileNotFound,
    #[value = 0x0002]
    InvalidExecutable,
    #[value = 0x0003]
    AccessDenied,
    #[value = 0xFFFF]
    Failure,
    #[fallback]
    Other(u16),
}

impl fmt::Display for ExecStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecStatusCode::Success => write!(f, "execution success"),
            ExecStatusCode::Failure => write!(f, "execution failed"),
            ExecStatusCode::FileNotFound => write!(f, "file not found"),
            ExecStatusCode::InvalidExecutable => write!(f, "invalid executable"),
            ExecStatusCode::AccessDenied => write!(f, "access denied"),
            ExecStatusCode::Other(code) => write!(f, "exec status code {}", code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_conversion() {
        let status = SeverityLevel::from(3u8);
        match status {
            SeverityLevel::Fatal => { /* success */ }
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
    fn parse_unknown_code() {
        let code = ExecStatusCode::from(327);
        assert!(matches!(code, ExecStatusCode::Other(327)));
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
