use alloc::borrow::Cow;
use alloc::fmt;
use core::convert::TryInto;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NoStdIoErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Other,
    UnexpectedEof,
    Unknown,
}

#[cfg(feature = "std")]
impl From<std::io::ErrorKind> for NoStdIoErrorKind {
    fn from(e: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind;
        match e {
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::PermissionDenied => Self::PermissionDenied,
            ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            ErrorKind::ConnectionReset => Self::ConnectionReset,
            ErrorKind::ConnectionAborted => Self::ConnectionAborted,
            ErrorKind::NotConnected => Self::NotConnected,
            ErrorKind::AddrInUse => Self::AddrInUse,
            ErrorKind::AddrNotAvailable => Self::AddrNotAvailable,
            ErrorKind::BrokenPipe => Self::BrokenPipe,
            ErrorKind::AlreadyExists => Self::AlreadyExists,
            ErrorKind::WouldBlock => Self::WouldBlock,
            ErrorKind::InvalidInput => Self::InvalidInput,
            ErrorKind::InvalidData => Self::InvalidData,
            ErrorKind::TimedOut => Self::TimedOut,
            ErrorKind::WriteZero => Self::WriteZero,
            ErrorKind::Interrupted => Self::Interrupted,
            ErrorKind::Other => Self::Other,
            ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NoStdIoError {
    kind: NoStdIoErrorKind,
    desc: Option<Cow<'static, str>>,
}

impl fmt::Display for NoStdIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.desc {
            Some(desc) => write!(f, "{:?} ({})", self.kind, desc),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NoStdIoError {}

#[cfg(feature = "std")]
impl From<std::io::Error> for NoStdIoError {
    fn from(e: std::io::Error) -> Self {
        Self::new_with_desc(e.kind().into(), e.to_string())
    }
}

impl NoStdIoError {
    pub const fn new(kind: NoStdIoErrorKind) -> Self {
        Self { kind, desc: None }
    }

    pub fn new_with_desc(kind: NoStdIoErrorKind, desc: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            desc: Some(desc.into()),
        }
    }

    pub const fn kind(&self) -> NoStdIoErrorKind {
        self.kind
    }

    pub fn desc(&self) -> Option<&str> {
        self.desc.as_deref()
    }
}

/// Subset of `std::io::Write` for no_std support
pub trait NoStdWrite {
    fn write(&mut self, buf: &[u8]) -> Result<usize, NoStdIoError>;

    fn flush(&mut self) -> Result<(), NoStdIoError>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), NoStdIoError> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(NoStdIoError::new_with_desc(
                        NoStdIoErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == NoStdIoErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_u8(&mut self, n: u8) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u16(&mut self, n: u16) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u32(&mut self, n: u32) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_u64(&mut self, n: u64) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i8(&mut self, n: i8) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i16(&mut self, n: i16) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i32(&mut self, n: i32) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }

    fn write_i64(&mut self, n: i64) -> Result<(), NoStdIoError> {
        self.write_all(&n.to_le_bytes())
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Write> NoStdWrite for T {
    fn write(&mut self, buf: &[u8]) -> Result<usize, NoStdIoError> {
        std::io::Write::write(self, buf).map_err(NoStdIoError::from)
    }

    fn flush(&mut self) -> Result<(), NoStdIoError> {
        std::io::Write::flush(self).map_err(NoStdIoError::from)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), NoStdIoError> {
        std::io::Write::write_all(self, buf).map_err(NoStdIoError::from)
    }
}

#[cfg(not(feature = "std"))]
impl NoStdWrite for alloc::vec::Vec<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, NoStdIoError> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), NoStdIoError> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), NoStdIoError> {
        self.extend_from_slice(buf);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cursor<'a> {
    inner: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub const fn new(inner: &[u8]) -> Cursor<'_> {
        Cursor { inner, pos: 0 }
    }

    pub const fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    #[inline]
    pub fn peek_u8(&self) -> Result<u8, NoStdIoError> {
        let v = self
            .inner
            .get(self.pos)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        Ok(*v)
    }

    #[inline]
    pub fn peek_u16(&self) -> Result<u16, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 1)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        Ok(u16::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn peek_u32(&self) -> Result<u32, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 3)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        Ok(u32::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn peek_u64(&self) -> Result<u64, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 7)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        Ok(u64::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn peek_rest(&self) -> Result<&'a [u8], NoStdIoError> {
        self.inner
            .get(self.pos..)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))
    }

    #[inline]
    pub fn peek_n(&mut self, n: usize) -> Result<&'a [u8], NoStdIoError> {
        self.inner
            .get(self.pos..self.pos + n)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))
    }

    #[inline]
    pub fn get_ref(&self) -> &'a [u8] {
        &self.inner
    }

    #[inline]
    pub fn rewind(&mut self, len: usize) {
        self.pos -= len;
    }

    #[inline]
    pub fn forward(&mut self, len: usize) {
        self.pos += len;
    }

    #[inline]
    pub fn read_n(&mut self, n: usize) -> Result<&'a [u8], NoStdIoError> {
        let bytes = self
            .inner
            .get(self.pos..self.pos + n)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += n;
        Ok(bytes)
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, NoStdIoError> {
        let v = self
            .inner
            .get(self.pos)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 1;
        Ok(*v)
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 2)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 2;
        Ok(u16::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 4)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 4;
        Ok(u32::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 8)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 8;
        Ok(u64::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_i8(&mut self) -> Result<i8, NoStdIoError> {
        let v = self
            .inner
            .get(self.pos)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 1;
        Ok(*v as i8)
    }

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 2)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 2;
        Ok(i16::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 4)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 4;
        Ok(i32::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_i64(&mut self) -> Result<i64, NoStdIoError> {
        let range = self
            .inner
            .get(self.pos..self.pos + 8)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += 8;
        Ok(i64::from_le_bytes(range.try_into().unwrap()))
    }

    #[inline]
    pub fn read_rest(&mut self) -> Result<&'a [u8], NoStdIoError> {
        let rest = self
            .inner
            .get(self.pos..)
            .ok_or(NoStdIoError::new(NoStdIoErrorKind::UnexpectedEof))?;
        self.pos += rest.len();
        Ok(rest)
    }
}
