use proc_macro2::TokenStream;
use quote::quote;

use syn::{
    parse::{ParseStream, Parser},
    Error, LitStr, Result,
};

use crate::common::expect_t;

pub(crate) fn cstr16impl(tokens: TokenStream) -> TokenStream {
    cstr16parse
        .parse2(tokens)
        .unwrap_or_else(Error::into_compile_error)
}

fn cstr16parse(input: ParseStream) -> Result<TokenStream> {
    if input.is_empty() {
        return Ok(quote!(unsafe {
            ::uefi::types::CStr16::from_u16_unchecked(&[0u16])
        }));
    }

    // TODO:ucs2
    let lit_str: LitStr = expect_t(&input)?;
    let lit_string = lit_str.value();
    let utf16 = lit_string.encode_utf16();

    Ok(quote!(unsafe {
        ::uefi::types::CStr16::from_u16_unchecked(&[ #(#utf16 ,)* 0u16])
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_cstr16() {
        assert_eq!(
            cstr16impl(quote! {}).to_string(),
            quote! {
                unsafe { ::uefi::types::CStr16::from_u16_unchecked(&[0u16]) }
            }
            .to_string()
        );

        assert_eq!(
            cstr16impl(quote! {""}).to_string(),
            quote! {
                unsafe { ::uefi::types::CStr16::from_u16_unchecked(&[0u16]) }
            }
            .to_string()
        );

        assert_eq!(
            cstr16impl(quote! {"hello"}).to_string(),
            quote! {
                unsafe { ::uefi::types::CStr16::from_u16_unchecked(&[104u16,101u16,108u16,108u16,111u16,0u16]) }
            }
            .to_string()
        );
        [0];
    }
}
