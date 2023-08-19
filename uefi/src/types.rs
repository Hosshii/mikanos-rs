use core::{ffi::c_void, fmt::Display};

#[must_use]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Status(Uintn);

impl Status {
    const ERROR_BIT: Uintn = 1 << (core::mem::size_of::<Uintn>() * 8 - 1);

    pub const SUCCRSS: Self = Self(0);

    pub fn is_err(self) -> bool {
        self.0 & Self::ERROR_BIT != 0
    }

    pub fn is_success(self) -> bool {
        self == Self::SUCCRSS
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Handle = *mut Void;
pub type Uint8 = u8;
pub type Uint16 = u16;
pub type Uint32 = u32;
pub type Uint64 = u64;
pub type Uintn = usize;
pub type Int16 = i16;
pub type Int32 = i32;
pub type Int64 = i64;
pub type Bool = bool;
pub type Void = c_void;
pub type Event = *mut Void;

// 全部定義するのは面倒くさいので、使わないポインタはこれで代用する
pub(crate) type UnusedPtr = *const usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Char16(u16);
impl TryFrom<char> for Char16 {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if (value as u32) < 0xffff {
            Ok(Char16(value as u16))
        } else {
            Err(())
        }
    }
}

pub const NULL_16: Char16 = Char16(0);

#[repr(transparent)]
pub struct CStr16 {
    innner: [Char16],
}

impl CStr16 {
    pub fn as_ptr(&self) -> *const Char16 {
        self.innner.as_ptr()
    }

    pub fn from_str_with_buf<'a>(input: &str, buf: &'a mut [u16]) -> Result<&'a Self, ()> {
        let mut idx = 0;
        for c in input.encode_utf16() {
            let Some(slot) = buf.get_mut(idx) else {
                return Err(());
            };
            if c == 0 && idx != input.len() {
                return Err(());
            }
            *slot = c;
            idx += 1;
        }
        *buf.get_mut(idx).ok_or(())? = 0;

        Ok(unsafe { Self::from_u16_unchecked(buf) })
    }

    pub unsafe fn from_u16_unchecked(v: &[u16]) -> &Self {
        unsafe { &*(v as *const [u16] as *const CStr16) }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(C, align(64))]
pub struct Guid {
    data_1: u32,
    data_2: u16,
    data_3: u16,
    data_4: [u8; 8],
}

impl Guid {
    pub(crate) const fn new(data_1: u32, data_2: u16, data_3: u16, data_4: [u8; 8]) -> Self {
        Self {
            data_1,
            data_2,
            data_3,
            data_4,
        }
    }
}

#[repr(C)]
pub struct Time {
    pub year: Uint16,
    pub month: Uint8,
    pub day: Uint8,
    pub hour: Uint8,
    pub minute: Uint8,
    pub second: Uint8,
    _pad1: Uint8,
    pub nano_second: Uint8,
    pub tize_zone: Int16,
    pub day_light: Uint8,
    _pad2: Uint8,
}
