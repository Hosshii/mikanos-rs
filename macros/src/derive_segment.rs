use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    Data, DataStruct, DeriveInput, Error, Expr, ExprLit, Fields, Generics, Ident, Lit, Result,
    Type, TypeArray, TypePath,
};

pub(crate) fn from_segment_impl(tokens: TokenStream) -> Result<TokenStream> {
    gen_from_segment.parse2(tokens)
}

pub(crate) fn into_segment_impl(tokens: TokenStream) -> Result<TokenStream> {
    gen_into_segment.parse2(tokens)
}

fn parse_input(input: ParseStream) -> Result<(Ident, Generics, Fields)> {
    let span = input.span();
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input.parse::<DeriveInput>()?;

    let Data::Struct(DataStruct { fields, .. }) = data else {
        return Err(Error::new(span, "input must be struct"));
    };

    Ok((ident, generics, fields))
}

fn gen_from_segment(input: ParseStream) -> Result<TokenStream> {
    let (struct_name, generics, fields) = parse_input(input)?;

    let (field_ty, _, _) = get_segment_type(&fields)?;
    let len = fields.len();
    let idx = 0..len;
    let field_name = fields.iter().map(|v| v.ident.as_ref().unwrap());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics FromSegment<#len> for #struct_name #ty_generics #where_clause {
            type Element = #field_ty;

            fn from_segment(v: [Self::Element; #len]) -> Self {
                Self {
                    #(
                        #field_name: v[#idx],
                    )*
                }
            }
        }
    })
}

fn gen_into_segment(input: ParseStream) -> Result<TokenStream> {
    let (struct_name, generics, fields) = parse_input(input)?;

    let (field_ty, _, _) = get_segment_type(&fields)?;
    let len = fields.len();
    let field_name = fields.iter().map(|v| v.ident.as_ref().unwrap());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics IntoSegment<#len> for #struct_name #ty_generics #where_clause {
            type Element = #field_ty;

            fn into_segment(self) -> [Self::Element; #len] {
                [
                    #(
                        self.#field_name,
                    )*
                ]
            }
        }
    })
}

fn get_segment_type(fields: &Fields) -> Result<(&Type, &Ident, u32)> {
    let size = fields
        .iter()
        .map(|v| get_ty_ident(&v.ty))
        .collect::<Result<Vec<(&Type, &Ident, u32)>>>()?;

    match size.as_slice() {
        [head, tail @ ..] => {
            if tail.iter().all(|x| x.1 == head.1) {
                let size: u32 = size.iter().map(|v| v.2).sum();
                Ok((head.0, head.1, size))
            } else {
                Err(Error::new_spanned(fields, "fields type does not equal"))
            }
        }
        [] => Err(Error::new_spanned(fields, "fields type does not equal")),
    }
}

/// 普通の型はその型野中絵を取得する。
/// 配列はその要素型を取得する
/// 型が何個かも取得する
fn get_ty_ident(ty: &Type) -> Result<(&Type, &Ident, u32)> {
    match &ty {
        Type::Array(TypeArray {
            elem,
            len:
                Expr::Lit(ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }),
            ..
        }) => {
            let len = lit_int.base10_parse::<u32>()?;
            let (ty, ident, size) = get_ty_ident(elem)?;

            Ok((ty, ident, size * len))
        }
        Type::Path(TypePath { path, .. }) => {
            let ident = path
                .get_ident()
                .ok_or(Error::new_spanned(ty, "single type required"))?;
            Ok((ty, ident, 1))
        }
        _ => Err(Error::new_spanned(ty, "type should be array or integer")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn test_from_segment() {
        assert_str_eq!(
            from_segment_impl(quote! {
                struct A {
                    a: u8,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl FromSegment<1usize> for A {
                    type Element = u8;

                    fn from_segment(v: [Self::Element; 1usize]) -> Self {
                        Self {
                            a: v[0usize],
                        }
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            from_segment_impl(quote! {
                struct A {
                    a: u32,
                    b: u32,
                    c: u32,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl FromSegment<3usize> for A {
                    type Element = u32;

                    fn from_segment(v: [Self::Element; 3usize]) -> Self {
                        Self {
                            a: v[0usize],
                            b: v[1usize],
                            c: v[2usize],
                        }
                    }
                }
            }
            .to_string()
        );

        assert!(from_segment_impl(quote! {
            struct A {}
        })
        .is_err());

        assert!(from_segment_impl(quote! {
            struct A {
                a: u32,
                b: u8,
            }
        })
        .is_err());
    }

    #[test]
    fn test_into_segment() {
        assert_str_eq!(
            into_segment_impl(quote! {
                struct A {
                    a: u8,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl IntoSegment<1usize> for A {
                    type Element = u8;

                    fn into_segment(self) -> [Self::Element; 1usize] {
                        [
                            self.a,
                        ]
                    }
                }
            }
            .to_string()
        );

        assert_str_eq!(
            into_segment_impl(quote! {
                struct A {
                    a: u32,
                    b: u32,
                    c: u32,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                impl IntoSegment<3usize> for A {
                    type Element = u32;

                    fn into_segment(self) -> [Self::Element; 3usize] {
                        [
                            self.a,
                            self.b,
                            self.c,
                        ]
                    }
                }
            }
            .to_string()
        );

        assert!(into_segment_impl(quote! {
            struct A {}
        })
        .is_err());

        assert!(into_segment_impl(quote! {
            struct A {
                a: u32,
                b: u8,
            }
        })
        .is_err());
    }
}
