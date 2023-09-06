pub trait Endian {
    fn from_le(v: Self) -> Self;
    fn from_be(v: Self) -> Self;

    fn to_le(self) -> Self;
    fn to_be(self) -> Self;
}

macro_rules! impl_endian {
    ($($type:ty),*) => {
        $(
            impl Endian for $type {
                fn from_le(v: Self) -> Self {
                    <$type>::from_le(v)
                }

                fn from_be(v: Self) -> Self {
                    <$type>::from_be(v)
                }

                fn to_le(self) -> Self {
                    self.to_le()
                }

                fn to_be(self) -> Self {
                    self.to_be()
                }
            }
        )*
    };
}

impl_endian!(u8, u16, u32, u64, u128, i8, i32, i64, i128);

macro_rules! declare_endian_be {
    ($($type:ty,$id:ident);* $(;)?) => {
        pub use declare_endian_be_mod::*;

        mod declare_endian_be_mod {
            $(
                #[repr(transparent)]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub struct $id($type);

                impl $id {
                    pub fn new(v: $type) -> Self {
                        Self(v.to_be())
                    }

                    pub fn from_unchecked(v: $type) -> Self {
                        Self(v)
                    }
                }

                impl From<$type> for $id {
                    fn from(v: $type) -> $id {
                        $id::new(v)
                    }
                }
            )*
        }
    };
}

macro_rules! declare_endian_le {
    ($($type:ty,$id:ident);* $(;)?) => {
        pub use declare_endian_le_mod::*;

        mod declare_endian_le_mod {
            $(
                #[repr(transparent)]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub struct $id($type);

                impl $id {
                    pub fn new(v: $type) -> Self {
                        Self(v.to_le())
                    }

                    pub fn from_unchecked(v: $type) -> Self {
                        Self(v)
                    }
                }

                impl From<$type> for $id {
                    fn from(v: $type) -> $id {
                        $id::new(v)
                    }
                }
            )*
        }
    };
}

declare_endian_be!(
    u8,  BeU8;
    u16, BeU16;
    u32, BeU32;
    u64, BeU64;
);

declare_endian_le!(
    u8,  LeU8;
    u16, LeU16;
    u32, LeU32;
    u64, LeU64;
);
