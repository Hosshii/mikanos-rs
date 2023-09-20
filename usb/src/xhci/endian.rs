pub trait EndianFrom<T = Self> {
    fn from_le(v: T) -> Self;
    fn from_be(v: T) -> Self;
    fn from_ne(v: T) -> Self;
}

pub trait EndianInto<T = Self> {
    fn to_le(self) -> T;
    fn to_be(self) -> T;
    fn to_ne(self) -> T;
}

pub trait Endian<T = Self>: EndianFrom<T> + EndianInto<T> {}
impl<T> Endian<T> for T where T: EndianFrom<T> + EndianInto<T> {}

impl EndianFrom for bool {
    fn from_le(v: Self) -> Self {
        v
    }

    fn from_be(v: Self) -> Self {
        v
    }

    fn from_ne(v: Self) -> Self {
        v
    }
}

impl EndianInto for bool {
    fn to_le(self) -> Self {
        self
    }

    fn to_be(self) -> Self {
        self
    }

    fn to_ne(self) -> Self {
        self
    }
}

macro_rules! impl_endian {
    ($($type:ty),*) => {
        $(
            impl EndianFrom for $type {
                fn from_le(v: Self) -> Self {
                    <$type>::from_le(v)
                }

                fn from_be(v: Self) -> Self {
                    <$type>::from_be(v)
                }

                fn from_ne(v: Self) -> Self {
                    v
                }
            }

            impl EndianInto for $type {
                fn to_le(self) -> Self {
                    self.to_le()
                }

                fn to_be(self) -> Self {
                    self.to_be()
                }

                fn to_ne(self) -> Self {
                    self
                }
            }
        )*
    };
}

impl_endian!(u8, u16, u32, u64, u128, i8, i32, i64, i128);
