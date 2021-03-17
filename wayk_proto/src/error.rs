use crate::message::{ChannelName, MessageType};
use crate::sharee::ShareeState;
use crate::sm::ConnectionState;
use core::fmt;
use core::num::TryFromIntError;

pub type Result<T> = core::result::Result<T, ProtoError>;

#[derive(Debug)]
pub struct ProtoError {
    pub kind: ProtoErrorKind,
    pub description: Option<alloc::borrow::Cow<'static, str>>,
    pub source: Option<alloc::boxed::Box<ProtoError>>,
}

sa::assert_impl_all!(ProtoError: Sync, Send);

impl ProtoError {
    pub fn new<T>(kind: ProtoErrorKind) -> core::result::Result<T, ProtoError> {
        Err(Self::from(kind))
    }

    pub fn into_source(self) -> Option<Self> {
        self.source.map(|boxed| *boxed)
    }

    #[cfg(feature = "std")]
    pub fn print_trace(&self) {
        print!("–– Error trace: ");
        self.__print_trace();
        println!();
    }

    #[cfg(feature = "std")]
    fn __print_trace(&self) {
        print!("{}", self.kind);

        if let Some(desc) = &self.description {
            print!(" [description: {}]", desc);
        }

        if let Some(source) = &self.source {
            print!("\n\t↳ source: ");
            source.__print_trace();
        }
    }
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        if let Some(desc) = &self.description {
            write!(f, " [description: {}]", desc)?;
        }

        if let Some(source) = &self.source {
            write!(f, " [source: {}]", source)?;
        }

        Ok(())
    }
}

pub trait ProtoErrorResultExt<T>
where
    Self: core::marker::Sized,
{
    #[allow(unused_variables)]
    fn or_else_desc<F, S>(self, f: F) -> core::result::Result<T, ProtoError>
    where
        F: FnOnce() -> S,
        S: Into<alloc::borrow::Cow<'static, str>>,
    {
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn or_desc<S>(self, desc: S) -> core::result::Result<T, ProtoError>
    where
        S: Into<alloc::borrow::Cow<'static, str>>,
    {
        unimplemented!()
    }

    #[allow(unused_variables)]
    fn source(self, src: ProtoError) -> core::result::Result<T, ProtoError> {
        unimplemented!()
    }

    fn chain(self, kind: ProtoErrorKind) -> core::result::Result<T, ProtoError>;
}

impl<T> ProtoErrorResultExt<T> for core::result::Result<T, ProtoError> {
    fn or_else_desc<F, S>(self, f: F) -> core::result::Result<T, ProtoError>
    where
        F: FnOnce() -> S,
        S: Into<alloc::borrow::Cow<'static, str>>,
    {
        match self {
            Err(_) => self.or_desc(f()),
            Ok(_) => self,
        }
    }

    fn or_desc<S>(self, desc: S) -> core::result::Result<T, ProtoError>
    where
        S: Into<alloc::borrow::Cow<'static, str>>,
    {
        self.map_err(|err| {
            let new_desc = if let Some(current_desc) = err.description {
                format!("{} [{}]", desc.into(), current_desc).into()
            } else {
                desc.into()
            };

            ProtoError {
                description: Some(new_desc),
                ..err
            }
        })
    }

    fn source(self, src: ProtoError) -> core::result::Result<T, ProtoError> {
        self.map_err(|err| ProtoError {
            source: Some(alloc::boxed::Box::new(src)),
            ..err
        })
    }

    fn chain(self, kind: ProtoErrorKind) -> core::result::Result<T, ProtoError> {
        self.map_err(|err| ProtoError {
            kind,
            description: None,
            source: Some(alloc::boxed::Box::new(err)),
        })
    }
}

impl<T> ProtoErrorResultExt<T> for Option<T> {
    fn chain(self, kind: ProtoErrorKind) -> core::result::Result<T, ProtoError> {
        self.ok_or_else(|| ProtoError::from(kind))
    }
}

impl From<ProtoErrorKind> for ProtoError {
    fn from(kind: ProtoErrorKind) -> Self {
        Self {
            kind,
            description: None,
            source: None,
        }
    }
}

impl From<crate::io::NoStdIoError> for ProtoError {
    fn from(e: crate::io::NoStdIoError) -> Self {
        Self::from(ProtoErrorKind::Io(e))
    }
}

impl From<alloc::string::FromUtf8Error> for ProtoError {
    fn from(e: alloc::string::FromUtf8Error) -> Self {
        Self::from(ProtoErrorKind::FromUtf8(e))
    }
}

impl From<core::num::TryFromIntError> for ProtoError {
    fn from(e: core::num::TryFromIntError) -> Self {
        Self::from(ProtoErrorKind::IntConversion(e))
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for ProtoError {
    fn from(e: std::io::Error) -> Self {
        Self::from(ProtoErrorKind::Io(crate::io::NoStdIoError::from(e)))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ProtoErrorKind {
    Decoding(&'static str),
    Encoding(&'static str),
    ConnectionSequence(ConnectionState),
    VirtualChannel(ChannelName),
    ChannelsManager,
    UnexpectedMessage(MessageType),
    Sharee(ShareeState),
    Io(crate::io::NoStdIoError),
    FromUtf8(alloc::string::FromUtf8Error),
    IntConversion(TryFromIntError),
}

impl fmt::Display for ProtoErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtoErrorKind::Decoding(desc) => write!(f, "couldn't decode {}", desc),
            ProtoErrorKind::Encoding(desc) => write!(f, "couldn't encode {}", desc),
            ProtoErrorKind::ConnectionSequence(state) => write!(f, "connection sequence failed at state {:?}", state),
            ProtoErrorKind::VirtualChannel(name) => write!(f, "virtual channel {:?} failed", name),
            ProtoErrorKind::ChannelsManager => write!(f, "virtual channels manager failed"),
            ProtoErrorKind::UnexpectedMessage(packet) => write!(f, "unexpected {:?} message", packet),
            ProtoErrorKind::Sharee(state) => write!(f, "sharee error in state {:?}", state),
            ProtoErrorKind::Io(e) => write!(f, "io error: {}", e),
            ProtoErrorKind::FromUtf8(e) => write!(f, "couldn't parse utf8 string: {}", e),
            ProtoErrorKind::IntConversion(e) => write!(f, "integer conversion failed: {}", e),
        }
    }
}
