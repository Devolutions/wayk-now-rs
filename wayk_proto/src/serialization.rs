use crate::error::ProtoError;
use crate::io::{Cursor, NoStdWrite};
use alloc::boxed::Box;
use alloc::vec::Vec;

// === ENCODE ===

pub enum ExpectedSize {
    Known(usize),
    Variable,
}

/// Common interface for encoding
pub trait Encode {
    fn expected_size() -> ExpectedSize
    where
        Self: Sized;

    fn encoded_len(&self) -> usize;

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError>
    where
        Self: Sized;

    fn encode(&self) -> Result<Vec<u8>, ProtoError>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        self.encode_into(&mut buf)?;
        Ok(buf)
    }
}

sa::assert_obj_safe!(Encode);

// === DECODE ===

/// Common interface for decoding
///
/// `'dec` lifetime **should not** appear in the type to which
/// the `Encode` impl applies.
///
/// Types that borrows **should implement the trait like this**:
/// ```
/// use wayk_proto::serialization::Decode;
/// use wayk_proto::error::ProtoError;
/// use wayk_proto::io::Cursor;
///
/// // my type that borrows
/// struct MyType<'a> {
///     data: &'a [u8],
/// }
///
/// impl<'dec: 'a, 'a> Decode<'dec> for MyType<'a> {
///     fn decode_from(cursor: &mut Cursor<'dec>) -> Result<Self, ProtoError> {
///         unimplemented!()
///     }
/// }
/// ```
/// That is, **do not do this**:
/// ```
/// use wayk_proto::serialization::Decode;
/// use wayk_proto::error::ProtoError;
/// use wayk_proto::io::Cursor;
///
/// struct MyType<'a> {
///     data: &'a [u8],
/// }
///
/// impl<'dec> Decode<'dec> for MyType<'dec> {
///     fn decode_from(cursor: &mut Cursor<'dec>) -> Result<Self, ProtoError> {
///         unimplemented!()
///     }
/// }
/// ```
/// Sooner or later it will explodes in your face.
pub trait Decode<'dec>
where
    Self: Sized,
{
    fn decode_from(cursor: &mut Cursor<'dec>) -> Result<Self, ProtoError>;

    fn decode(bytes: &'dec [u8]) -> Result<Self, ProtoError> {
        Self::decode_from(&mut Cursor::new(bytes))
    }
}

// === implementation for primitive types ===

impl Encode for u8 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u8(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for u8 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_u8().map_err(ProtoError::from)
    }
}

impl Encode for u16 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u16(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for u16 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_u16().map_err(ProtoError::from)
    }
}

impl Encode for u32 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u32(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for u32 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_u32().map_err(ProtoError::from)
    }
}

impl Encode for u64 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u64(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for u64 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_u64().map_err(ProtoError::from)
    }
}

impl Encode for i8 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_i8(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for i8 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_i8().map_err(ProtoError::from)
    }
}

impl Encode for i16 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_i16(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for i16 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_i16().map_err(ProtoError::from)
    }
}

impl Encode for i32 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_i32(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for i32 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_i32().map_err(ProtoError::from)
    }
}

impl Encode for i64 {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_i64(*self).map_err(ProtoError::from)
    }
}

impl Decode<'_> for i64 {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        cursor.read_i64().map_err(ProtoError::from)
    }
}

impl Encode for [u32; 4] {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Known(core::mem::size_of::<Self>())
    }

    fn encoded_len(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        for element in self {
            element.encode_into(writer)?;
        }
        Ok(())
    }
}

impl Decode<'_> for [u32; 4] {
    fn decode_from(cursor: &mut Cursor<'_>) -> Result<Self, ProtoError> {
        Ok([
            cursor.read_u32()?,
            cursor.read_u32()?,
            cursor.read_u32()?,
            cursor.read_u32()?,
        ])
    }
}

impl Encode for &[u8] {
    fn expected_size() -> ExpectedSize {
        ExpectedSize::Variable
    }

    fn encoded_len(&self) -> usize {
        self.len()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_all(self)?;
        Ok(())
    }
}

impl<'dec: 'a, 'a, T: 'a> Decode<'dec> for Box<T>
where
    T: Decode<'dec>,
{
    fn decode_from(cursor: &mut Cursor<'dec>) -> Result<Self, ProtoError> {
        T::decode_from(cursor).map(Box::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::Bytes8;

    #[derive(Encode, Decode)]
    struct StructDerive<'a> {
        pub a: u8,
        b: u8,
        pub c: u16,
        update_data: Bytes8<'a>,
    }

    const STRUCT_DERIVE_ENCODED: [u8; 8] = [0x10, 0x20, 0x30, 0x40, 0x03, 0x01, 0x02, 0x03];

    #[test]
    fn struct_derive_decode() {
        let s = StructDerive::decode(&STRUCT_DERIVE_ENCODED).unwrap();
        assert_eq!(s.a, 0x10);
        assert_eq!(s.b, 0x20);
        assert_eq!(s.c, 0x4030);
        assert_eq!(s.update_data, &[0x01, 0x02, 0x03][0..]);
    }

    #[test]
    fn struct_derive_encode() {
        let s = StructDerive {
            a: 0x10,
            b: 0x20,
            c: 0x4030,
            update_data: Bytes8(&[0x01, 0x02, 0x03]),
        };
        assert_eq!(s.encode().unwrap(), STRUCT_DERIVE_ENCODED.to_vec());
    }
}
