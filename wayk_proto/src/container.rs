macro_rules! impl_container {
    ($ty:ident as Vec with $size_ty:ident) => {
        #[derive(PartialEq, Debug, Clone)]
        pub struct $ty<Item>(pub ::alloc::vec::Vec<Item>);

        impl<Item> ::core::ops::Deref for $ty<Item> {
            type Target = ::alloc::vec::Vec<Item>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<Item> ::core::ops::DerefMut for $ty<Item> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<Item> ::core::iter::IntoIterator for $ty<Item> {
            type Item = Item;
            type IntoIter = ::alloc::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a, Item> ::core::iter::IntoIterator for &'a $ty<Item> {
            type Item = &'a Item;
            type IntoIter = ::alloc::slice::Iter<'a, Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a, Item> ::core::iter::IntoIterator for &'a mut $ty<Item> {
            type Item = &'a mut Item;
            type IntoIter = ::alloc::slice::IterMut<'a, Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }

        impl<Item> From<::alloc::vec::Vec<Item>> for $ty<Item> {
            fn from(v: ::alloc::vec::Vec<Item>) -> Self {
                Self(v)
            }
        }

        impl<Item> Into<::alloc::vec::Vec<Item>> for $ty<Item> {
            fn into(self) -> ::alloc::vec::Vec<Item> {
                self.0
            }
        }

        impl<Item> PartialEq<::alloc::vec::Vec<Item>> for $ty<Item>
        where
            Item: PartialEq,
        {
            fn eq(&self, other: &::alloc::vec::Vec<Item>) -> bool {
                self.0.eq(other)
            }
        }

        impl<Item> $crate::serialization::Encode for $ty<Item>
        where
            Item: $crate::serialization::Encode + ::core::fmt::Debug,
        {
            fn expected_size() -> crate::serialization::ExpectedSize
            where
                Self: Sized,
            {
                crate::serialization::ExpectedSize::Variable
            }

            fn encoded_len(&self) -> usize {
                self.iter().fold(::core::mem::size_of::<$size_ty>(), |acc, item| {
                    acc + item.encoded_len()
                })
            }

            fn encode_into<W: $crate::io::NoStdWrite>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), $crate::error::ProtoError> {
                use ::core::convert::TryFrom;
                use $crate::error::*;

                let count = <$size_ty>::try_from(self.len())
                    .map_err($crate::error::ProtoError::from)
                    .chain($crate::error::ProtoErrorKind::Encoding(stringify!($ty)))
                    .or_desc("couldn't convert losslessly vec size into u8 (count)")?;
                count.encode_into(writer)?;
                for item in self {
                    item.encode_into(writer)
                        .chain($crate::error::ProtoErrorKind::Encoding(stringify!($ty)))
                        .or_else_desc(|| format!("couldn't encode item {:?}", item))?;
                }
                Ok(())
            }
        }

        impl<'dec, Item> $crate::serialization::Decode<'dec> for $ty<Item>
        where
            Item: $crate::serialization::Decode<'dec>,
        {
            fn decode_from(cursor: &mut $crate::io::Cursor<'dec>) -> Result<Self, $crate::error::ProtoError> {
                use $crate::error::*;

                let count = <$size_ty>::decode_from(cursor)
                    .chain($crate::error::ProtoErrorKind::Decoding(stringify!($ty)))
                    .or_desc("couldn't decode list count")?;
                let mut vec = ::alloc::vec::Vec::new();
                for i in 0..count {
                    vec.push(
                        Item::decode_from(cursor)
                            .chain($crate::error::ProtoErrorKind::Decoding(stringify!($ty)))
                            .or_else_desc(|| format!("couldn't decode item n°{}", i))?,
                    );
                }
                Ok(Self(vec))
            }
        }
    };
    ($ty:ident as &[u8] with $size_ty:ident) => {
        #[derive(PartialEq, Debug, Clone)]
        pub struct $ty<'a>(pub &'a [u8]);

        impl<'a> ::core::ops::Deref for $ty<'a> {
            type Target = &'a [u8];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> ::core::iter::IntoIterator for &'a $ty<'a> {
            type Item = &'a u8;
            type IntoIter = ::alloc::slice::Iter<'a, u8>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a> From<&'a [u8]> for $ty<'a> {
            fn from(v: &'a [u8]) -> Self {
                Self(v)
            }
        }

        impl<'a> Into<&'a [u8]> for $ty<'a> {
            fn into(self) -> &'a [u8] {
                self.0
            }
        }

        impl<'a> PartialEq<&'a [u8]> for $ty<'a> {
            fn eq(&self, other: &&'a [u8]) -> bool {
                self.0.eq(*other)
            }
        }

        impl $crate::serialization::Encode for $ty<'_> {
            fn expected_size() -> crate::serialization::ExpectedSize
            where
                Self: Sized,
            {
                crate::serialization::ExpectedSize::Variable
            }

            fn encoded_len(&self) -> usize {
                ::core::mem::size_of::<$size_ty>() + ::core::mem::size_of::<u8>() * self.len()
            }

            fn encode_into<W: $crate::io::NoStdWrite>(&self, writer: &mut W) -> Result<(), $crate::error::ProtoError> {
                use ::core::convert::TryFrom;
                use $crate::error::*;

                let count = <$size_ty>::try_from(self.len())
                    .map_err(ProtoError::from)
                    .chain($crate::error::ProtoErrorKind::Encoding(stringify!($ty)))
                    .or_else_desc(|| {
                        format!(
                            "couldn't convert losslessly slice size into {} (count)",
                            stringify!($size_ty)
                        )
                    })?;
                count.encode_into(writer)?;
                writer.write_all(self.0)?;
                Ok(())
            }
        }

        impl<'dec: 'a, 'a> $crate::serialization::Decode<'dec> for $ty<'a> {
            fn decode_from(cursor: &mut $crate::io::Cursor<'dec>) -> Result<Self, $crate::error::ProtoError> {
                use $crate::error::*;

                let count = <$size_ty>::decode_from(cursor)
                    .chain($crate::error::ProtoErrorKind::Decoding(stringify!($ty)))
                    .or_desc("couldn't decode list count")?;
                let start_inclusive = cursor.position() as usize;
                let slices_to_end = &cursor.get_ref()[start_inclusive..];
                if slices_to_end.len() < count as usize {
                    return ProtoError::new(ProtoErrorKind::Decoding(stringify!($ty))).or_else_desc(|| {
                        format!(
                            "couldn't decode list: count ({}) greater than available bytes ({})",
                            count,
                            slices_to_end.len()
                        )
                    });
                }
                let bytes = &slices_to_end[..count as usize];
                Ok($ty(bytes))
            }
        }
    };
}

impl_container! { Vec8  as Vec with u8  }
impl_container! { Vec16 as Vec with u16 }
impl_container! { Vec32 as Vec with u32 }
impl_container! { Vec64 as Vec with u64 }

impl_container! { Bytes8  as &[u8] with u8  }
impl_container! { Bytes16 as &[u8] with u16 }
impl_container! { Bytes32 as &[u8] with u32 }
impl_container! { Bytes64 as &[u8] with u64 }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    const U16_VEC8: [u8; 7] = [0x03, 0x50, 0x10, 0x0a, 0x09, 0x57, 0x0b];

    #[test]
    fn encode_vec8() {
        let vec = Vec8(vec![0x1050u16, 0x090au16, 0x0b57u16]);
        assert_eq!(vec.encode().unwrap(), &U16_VEC8);
    }

    #[test]
    fn decode_vec8() {
        assert_eq!(
            Vec8::<u16>::decode(&U16_VEC8).unwrap(),
            vec![0x1050u16, 0x090au16, 0x0b57u16]
        );
    }

    const U16_VEC32: [u8; 10] = [0x03, 0x00, 0x00, 0x00, 0x50, 0x10, 0x0a, 0x09, 0x57, 0x0b];

    #[test]
    fn encode_vec32() {
        let vec = Vec32(vec![0x1050u16, 0x090au16, 0x0b57u16]);
        assert_eq!(vec.encode().unwrap(), &U16_VEC32);
    }

    #[test]
    fn decode_vec32() {
        assert_eq!(
            Vec32::<u16>::decode(&U16_VEC32).unwrap(),
            vec![0x1050u16, 0x090au16, 0x0b57u16]
        );
    }

    const ENCODED_MSG_WITH_BYTES8: [u8; 13] = [
        0x38, 0xae, 0xf3, // things
        0x06, // count
        0x50, 0x10, 0x0a, 0x09, 0x57, 0x0b, // elements
        0xc3, 0xaf, 0x13, // other things
    ];

    #[test]
    fn encode_bytes8() {
        let slice = Bytes8(&ENCODED_MSG_WITH_BYTES8[4..=9]);
        assert_eq!(slice.encode().unwrap(), &ENCODED_MSG_WITH_BYTES8[3..=9]);
    }

    #[test]
    fn decode_bytes8() {
        assert_eq!(
            Bytes8::decode(&ENCODED_MSG_WITH_BYTES8[3..]).unwrap(),
            &ENCODED_MSG_WITH_BYTES8[4..=9]
        );
    }

    const ENCODED_MSG_WITH_BYTES32: [u8; 16] = [
        0x38, 0xae, 0xf3, // things
        0x06, 0x00, 0x00, 0x00, // count
        0x50, 0x10, 0x0a, 0x09, 0x57, 0x0b, // elements
        0xc3, 0xaf, 0x13, // other things
    ];

    #[test]
    fn encode_bytes32() {
        let slice = Bytes32(&ENCODED_MSG_WITH_BYTES32[7..=12]);
        assert_eq!(slice.encode().unwrap(), &ENCODED_MSG_WITH_BYTES32[3..=12]);
    }

    #[test]
    fn decode_bytes32() {
        assert_eq!(
            Bytes32::decode(&ENCODED_MSG_WITH_BYTES32[3..]).unwrap(),
            &ENCODED_MSG_WITH_BYTES32[7..=12]
        );
    }
}
