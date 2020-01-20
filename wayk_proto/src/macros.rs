// These macro can be used to generate boilerplate code related to wayknow packets.
// Normal user isn't expected to uses it, but it can be useful to those implementing
// protocol extensions (such as authentication tokens encoding/decoding).

// === FLAGS ===

#[doc(hidden)]
#[macro_export]
macro_rules! __flags_struct {
    ($flags_type:ident : $underlying_type:ident) => {
        #[derive(wayk_proto_derive::Encode, wayk_proto_derive::Decode, Debug, PartialEq, Clone, Copy)]
        pub struct $flags_type {
            pub value: $underlying_type,
        }

        impl From<$underlying_type> for $flags_type {
            fn from(value: $underlying_type) -> Self {
                Self { value }
            }
        }

        impl Into<$underlying_type> for $flags_type {
            fn into(self) -> $underlying_type {
                self.value
            }
        }

        impl PartialEq<$underlying_type> for $flags_type {
            fn eq(&self, other: &$underlying_type) -> bool {
                self.value == *other
            }
        }

        impl $flags_type {
            pub fn new_empty() -> Self {
                Self { value: 0 }
            }
        }
    };
    (
        $flags_type:ident : $underlying_type:ident => {
            $( $lowercase:ident = $UPPERCASE:ident = $const_value:expr , )+
        }
    ) => {
        $crate::__flags_struct!{ $flags_type : $underlying_type }

        impl $flags_type {
            $(
                pub const $UPPERCASE: $underlying_type = $const_value;
                pub fn $lowercase(self) -> bool {
                    self.value & Self::$UPPERCASE != 0
                }

                paste::item! {
                    pub fn [<set_ $lowercase>](&mut self) -> Self {
                        self.value |= Self::$UPPERCASE;
                        *self
                    }

                    pub fn [<unset_ $lowercase>](&mut self) -> Self {
                        self.value &= !Self::$UPPERCASE;
                        *self
                    }
                }
            )+
        }
    };
}
