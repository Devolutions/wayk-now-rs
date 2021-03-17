// NOW_STRING

use crate::error::*;
use crate::serialization::{Decode, Encode};
use alloc::borrow::Cow;
use byteorder::{ReadBytesExt, WriteBytesExt};
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::str::FromStr;
use std::io::{Cursor, Read, Write};

pub trait NowStringSize {
    const SIZE: usize;
}

macro_rules! now_string_size {
    ( $string_size_name:ident, $string_size_type:ident, $now_string_name:ident, $size:literal ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $string_size_name;

        impl NowStringSize for $string_size_name {
            const SIZE: usize = $size;
        }

        pub type $now_string_name = NowString<$string_size_name, $string_size_type>;
    };
}

now_string_size! { StringSize16,    u8,  NowString16,    16    }
now_string_size! { StringSize32,    u8,  NowString32,    32    }
now_string_size! { StringSize64,    u8,  NowString64,    64    }
now_string_size! { StringSize128,   u8,  NowString128,   128   }
now_string_size! { StringSize256,   u8,  NowString256,   256   }
now_string_size! { StringSize65535, u16, NowString65535, 65535 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowString<Size, SizeType> {
    inner: String,
    _pd: PhantomData<(Size, SizeType)>,
}

impl<'dec, Size, SizeType> Decode<'dec> for NowString<Size, SizeType>
where
    Size: NowStringSize,
    SizeType: Decode<'dec> + Into<usize>,
{
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let expected_size = SizeType::decode_from(cursor)?.into();

        if expected_size > Size::SIZE {
            return ProtoError::new(ProtoErrorKind::Decoding("NowString")).or_else_desc(|| {
                format!(
                    "attempted to parse a string greater (len: {}) than the NowString{} size limit",
                    expected_size,
                    Size::SIZE
                )
            });
        }

        let string = {
            let mut buffer = vec![0u8; expected_size];
            cursor
                .read_exact(&mut buffer)
                .map_err(ProtoError::from)
                .chain(ProtoErrorKind::Decoding("NowString"))
                .or_else_desc(|| {
                    format!(
                        "no enough bytes to parse the NowString{} (expected {})",
                        Size::SIZE,
                        expected_size
                    )
                })?;
            cursor.read_u8()?; // discard the null terminator

            String::from_utf8(buffer)
                .map_err(ProtoError::from)
                .chain(ProtoErrorKind::Decoding("NowString"))?
        };

        Ok(NowString {
            inner: string,
            _pd: PhantomData,
        })
    }
}

impl<Size, SizeType> Encode for NowString<Size, SizeType>
where
    SizeType: Encode + TryFrom<usize>,
    <SizeType as TryFrom<usize>>::Error: std::fmt::Debug,
{
    fn encoded_len(&self) -> usize {
        self.inner.len() + std::mem::size_of::<u8>() + std::mem::size_of::<SizeType>()
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        let bytes = self.inner.as_bytes();
        let len = SizeType::try_from(bytes.len()).unwrap(); // should never panic by construction
        len.encode_into(writer)?;
        bytes.encode_into(writer)?;
        writer.write_u8(0u8)?;
        Ok(())
    }
}

impl<Size, SizeType> NowString<Size, SizeType>
where
    Size: NowStringSize,
    SizeType: Encode + TryFrom<usize>,
    <SizeType as TryFrom<usize>>::Error: std::fmt::Debug,
{
    pub fn new_empty() -> Self {
        Self {
            inner: String::new(),
            _pd: PhantomData,
        }
    }

    /// # Safety
    /// Provided string len must not exceed `NowStringSize::SIZE`
    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self {
            inner: s,
            _pd: PhantomData,
        }
    }

    /// # Safety
    /// Provided string slice len must not exceed `NowStringSize::SIZE`
    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        Self {
            inner: s.to_string(),
            _pd: PhantomData,
        }
    }

    pub fn from_string(string: String) -> Result<Self> {
        Self::try_from(string)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Encode an utf8 str into a now string
    pub fn helper_write_into<W: Write>(writer: &mut W, s: &str) -> Result<()> {
        if s.len() > Size::SIZE {
            return ProtoError::new(ProtoErrorKind::Encoding("NowString")).or_else_desc(|| {
                format!(
                    "provided string greater (len: {}) than NowString{} size limit",
                    s.len(),
                    Size::SIZE
                )
            });
        }

        let len = SizeType::try_from(s.len()).unwrap(); // should never panic by construction
        len.encode_into(writer)?;
        writer.write_all(s.as_bytes())?;
        writer.write_u8(0u8)?; // null terminator
        Ok(())
    }
}

impl<Size, SizeType> Into<String> for NowString<Size, SizeType> {
    fn into(self) -> String {
        self.inner
    }
}

impl<'a, Size, SizeType> Into<Cow<'a, str>> for NowString<Size, SizeType> {
    fn into(self) -> Cow<'a, str> {
        self.inner.into()
    }
}

impl<Size, SizeType> TryFrom<String> for NowString<Size, SizeType>
where
    Size: NowStringSize,
{
    type Error = ProtoError;

    fn try_from(string: String) -> Result<Self> {
        if string.len() > Size::SIZE {
            return ProtoError::new(ProtoErrorKind::Decoding("NowString")).or_else_desc(|| {
                format!(
                    "provided string greater (len: {}) than NowString{} size limit",
                    string.len(),
                    Size::SIZE
                )
            });
        }

        Ok(Self {
            inner: string,
            _pd: PhantomData,
        })
    }
}

impl<Size, SizeType> FromStr for NowString<Size, SizeType>
where
    Size: NowStringSize,
{
    type Err = ProtoError;

    fn from_str(s: &str) -> Result<Self> {
        Self::try_from(s.to_string())
    }
}

impl<Size, SizeType> PartialEq<&str> for NowString<Size, SizeType> {
    fn eq(&self, other: &&str) -> bool {
        &self.inner == other
    }
}

impl<Size, SizeType> PartialEq<String> for NowString<Size, SizeType> {
    fn eq(&self, other: &String) -> bool {
        self.inner.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::Encode;

    const STRING_CHINESE: &'static str = "简介";

    #[rustfmt::skip]
    const NOW_STRING_CHINESE: [u8; 8] = [
        0x06, // size
        0xe7, 0xae, 0x80, 0xe4, 0xbb, 0x8b, // actual UTF8 string
        0x00, // null terminator
    ];

    #[test]
    fn decode_now_string_64() {
        let nstr = NowString64::decode(&NOW_STRING_CHINESE).unwrap();
        assert_eq!(nstr, STRING_CHINESE);
        assert_eq!(nstr.len(), 6);
        assert_eq!(nstr.encoded_len(), NOW_STRING_CHINESE.len());
    }

    #[test]
    fn decode_invalid_expected_size_now_string_64() {
        let result = NowString64::decode(&[0x08, 0xe7, 0xae, 0x80, 0xe4, 0xbb, 0x8b, 0x00]);
        let err = result.err().unwrap();
        assert_eq!(
            format!("{}", err),
            "couldn't decode NowString [description: no enough bytes to parse the NowString64 (expected 8)] [source: io error: failed to fill whole buffer]"
        );
    }

    #[test]
    fn decode_too_big_size_now_string_64() {
        let mut bytes = [0; 66];
        bytes[0] = 65;
        let result = NowString64::decode(&bytes);
        let err = result.err().unwrap();
        assert_eq!(
            format!("{}", err),
            "couldn't decode NowString [description: attempted to parse a string greater (len: 65) than the NowString64 size limit]"
        );
    }

    #[test]
    fn encode_now_string_64() {
        let nstr = NowString64::from_str(STRING_CHINESE).unwrap();
        assert_eq!(nstr.encode().unwrap(), NOW_STRING_CHINESE.to_vec());
    }

    #[test]
    fn now_string_64_helper() {
        let mut encoded_now_string = Vec::new();
        NowString64::helper_write_into(&mut encoded_now_string, STRING_CHINESE).unwrap();
        assert_eq!(encoded_now_string, NOW_STRING_CHINESE);
    }

    #[rustfmt::skip]
    const NOW_STRING_65535_CHINESE: [u8; 9] = [
        0x06, 0x00, // size
        0xe7, 0xae, 0x80, 0xe4, 0xbb, 0x8b, // actual UTF8 string
        0x00, // null terminator
    ];

    #[test]
    fn decode_now_string_65535() {
        let nstr = NowString65535::decode(&NOW_STRING_65535_CHINESE).unwrap();
        assert_eq!(nstr, STRING_CHINESE);
        assert_eq!(nstr.len(), 6);
        assert_eq!(nstr.encoded_len(), NOW_STRING_65535_CHINESE.len());
    }

    #[test]
    fn encode_now_string_65535() {
        let nstr = NowString65535::from_str(STRING_CHINESE).unwrap();
        assert_eq!(nstr.encode().unwrap(), NOW_STRING_65535_CHINESE.to_vec());
    }
}
