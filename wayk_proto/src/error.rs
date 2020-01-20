use crate::{
    message::{ChannelName, MessageType},
    sharee::ShareeState,
    sm::ConnectionState,
};
use core::{fmt, num::TryFromIntError};

macro_rules! error_chain {
    ($error_ty:ident, $error_kind_ty:ident) => {
        #[derive(Debug)]
        pub struct $error_ty {
            pub kind: $error_kind_ty,
            pub description: Option<std::borrow::Cow<'static, str>>,
            pub source: Option<Box<$error_ty>>,
        }

        impl $error_ty {
            pub fn new<T>(kind: $error_kind_ty) -> core::result::Result<T, $error_ty> {
                Err(Self::from(kind))
            }

            pub fn into_source(self) -> Option<Self> {
                self.source.map(|boxed| *boxed)
            }

            pub fn print_trace(&self) {
                print!("–– Error trace: ");
                self.__print_trace();
                println!("");
            }

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

        impl core::fmt::Display for $error_ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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

        paste::item! {
            pub trait [<$error_ty ResultExt>]<T>
            where
                Self: core::marker::Sized,
            {
                #[allow(unused_variables)]
                fn or_else_desc<F, S>(self, f: F) -> core::result::Result<T, $error_ty>
                where
                    F: FnOnce() -> S,
                    S: Into<std::borrow::Cow<'static, str>>,
                {
                    unimplemented!()
                }

                #[allow(unused_variables)]
                fn or_desc<S>(self, desc: S) -> core::result::Result<T, $error_ty>
                where
                    S: Into<std::borrow::Cow<'static, str>>,
                {
                    unimplemented!()
                }

                #[allow(unused_variables)]
                fn source(self, src: $error_ty) -> core::result::Result<T, $error_ty> {
                    unimplemented!()
                }

                fn chain(self, kind: $error_kind_ty) -> core::result::Result<T, $error_ty>;
            }

            impl<T> [<$error_ty ResultExt>]<T> for core::result::Result<T, $error_ty> {
                fn or_else_desc<F, S>(self, f: F) -> core::result::Result<T, $error_ty>
                where
                    F: FnOnce() -> S,
                    S: Into<std::borrow::Cow<'static, str>>,
                {
                    match self {
                        Err(_) => self.or_desc(f()),
                        Ok(_) => self,
                    }
                }

                fn or_desc<S>(self, desc: S) -> core::result::Result<T, $error_ty>
                where
                    S: Into<std::borrow::Cow<'static, str>>,
                {
                    self.map_err(|err| {
                        let new_desc = if let Some(current_desc) = err.description {
                            format!("{} [{}]", desc.into(), current_desc).into()
                        } else {
                            desc.into()
                        };

                        $error_ty {
                            description: Some(new_desc),
                            ..err
                        }
                    })
                }

                fn source(self, src: $error_ty) -> core::result::Result<T, $error_ty> {
                    self.map_err(|err| $error_ty {
                        source: Some(Box::new(src)),
                        ..err
                    })
                }

                fn chain(self, kind: $error_kind_ty) -> core::result::Result<T, $error_ty> {
                    self.map_err(|err| $error_ty {
                        kind,
                        description: None,
                        source: Some(Box::new(err)),
                    })
                }
            }

            impl<T> [<$error_ty ResultExt>]<T> for Option<T> {
                fn chain(self, kind: $error_kind_ty) -> core::result::Result<T, $error_ty> {
                    self.ok_or_else(|| $error_ty::from(kind))
                }
            }
        }

        impl From<$error_kind_ty> for $error_ty {
            fn from(kind: $error_kind_ty) -> Self {
                Self {
                    kind,
                    description: None,
                    source: None,
                }
            }
        }
    };
}

pub type Result<T> = std::result::Result<T, ProtoError>;

error_chain! { ProtoError, ProtoErrorKind }

sa::assert_impl_all!(ProtoError: Sync, Send);

impl From<std::io::Error> for ProtoError {
    fn from(e: std::io::Error) -> Self {
        Self::from(ProtoErrorKind::Io(e))
    }
}

impl From<std::string::FromUtf8Error> for ProtoError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::from(ProtoErrorKind::FromUtf8(e))
    }
}

impl From<std::num::TryFromIntError> for ProtoError {
    fn from(e: std::num::TryFromIntError) -> Self {
        Self::from(ProtoErrorKind::IntConversion(e))
    }
}

#[derive(Debug)]
pub enum ProtoErrorKind {
    Decoding(&'static str),
    Encoding(&'static str),
    ConnectionSequence(ConnectionState),
    VirtualChannel(ChannelName),
    ChannelsManager,
    UnexpectedMessage(MessageType),
    Sharee(ShareeState),
    Io(std::io::Error),
    FromUtf8(std::string::FromUtf8Error),
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
