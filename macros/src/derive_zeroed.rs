use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed, Result,
};

pub(crate) fn zeroed_impl(tokens: TokenStream) -> Result<TokenStream> {
    parse_input.parse2(tokens)
}

fn parse_input(input: ParseStream) -> Result<TokenStream> {
    let span = input.span();
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input.parse()?;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let Data::Struct(s) = data else {
        return Err(Error::new(span, "input must be struct"));
    };

    let token_stream = gen_struct(s);

    Ok(quote! {
        impl #impl_generics Zeroed for #ident #ty_generics #where_clause {
            fn zeroed() -> Self {
                #token_stream
            }
        }
    })
}

fn gen_struct(s: DataStruct) -> TokenStream {
    match s.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let inner: TokenStream = named
                .into_iter()
                .map(|Field { ty, ident, .. }| {
                    quote! {
                        #ident: <#ty>::zeroed(),
                    }
                })
                .collect();

            quote! {
                Self {
                    #inner
                }
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let inner: TokenStream = unnamed
                .into_iter()
                .map(|Field { ty, .. }| quote!(<#ty>::zeroed(),))
                .collect();
            quote! {
                Self(#inner)
            }
        }
        Fields::Unit => quote!(Self),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn test_zeroed() {
        assert_str_eq!(
            zeroed_impl(quote! {
                struct A {}
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self {}
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            zeroed_impl(quote! {
                struct A {
                    a: u8,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self {
                            a: <u8>::zeroed(),
                        }
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            zeroed_impl(quote! {
                struct A {
                    a: u8,
                    b: u32,
                    c: D,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self {
                            a: <u8>::zeroed(),
                            b: <u32>::zeroed(),
                            c: <D>::zeroed(),
                        }
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            zeroed_impl(quote! {
                struct A;
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            zeroed_impl(quote! {
                struct A();
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self()
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            zeroed_impl(quote! {
                struct A(u32, u16, D);
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl Zeroed for A {
                    fn zeroed() -> Self {
                        Self(<u32>::zeroed(), <u16>::zeroed(), <D>::zeroed(),)
                    }
                }
            }
            .to_string()
        );
    }
}
