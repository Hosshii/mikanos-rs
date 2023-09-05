use syn::Error;

mod cstr16;
mod reg;

#[proc_macro]
pub fn cstr16(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cstr16::cstr16impl(tokens.into()).into()
}

/// low | 15----------------06 05-------03 02----01 00--00
///     |        flag4           flag3      flag2   flag1
/// hi  | flag12 flag11 flag10 flag9 flag8 flag7 flag6 flag5
/// ```rust
/// #[bitfield_struct]
/// struct Hoge {
///     flag123: u16 => {
///         #[bits(1)]
///         flag1: bool,
///         #[bits(2)]
///         flag2: u8,
///         #[bits(3)]
///         flag3: u8,
///         #[bits(10)]
///         flag4: u16,
///     },
///     
///     flags: [u8; 2] => [
///         {
///             #[bits(2)]
///             flag5: u8,
///             #[bits(2)]
///             flag6: u8,
///             #[bits(2)]
///             flag7: u8,
///             #[bits(2)]
///             flag8: u8,
///         },
///         {
///             #[bits(2)]
///             flag9: u8,
///             #[bits(2)]
///             flag10: u8,
///             #[bits(2)]
///             flag11: u8,
///             #[bits(2)]
///             flag12: u8,
///         }
///     ]
/// }     
///
#[proc_macro]
pub fn bitfield_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    reg::bitfield_struct_impl(input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
