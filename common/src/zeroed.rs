pub use macros::Zeroed;

pub trait Zeroed {
    fn zeroed() -> Self;
}

macro_rules! impl_zeroed {
    ($($t:ty),*) => {
        $(
            impl Zeroed for $t {
                fn zeroed() -> Self {
                    0
                }
            }
        )*
    };
}

impl_zeroed!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl<T, const N: usize> Zeroed for [T; N]
where
    T: Zeroed + Copy,
{
    fn zeroed() -> Self {
        [T::zeroed(); N]
    }
}
