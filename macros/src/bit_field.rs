use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
// `remove unused imports` token::Bracket regardless it is used or not.
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, FatArrow, Semi, Struct},
    Attribute, Error, Expr, ExprLit, Field, FieldsNamed, Generics, Ident, Lit, LitInt, LitStr,
    Meta, MetaList, MetaNameValue, Result, Token, Type, TypeArray, TypePath, Visibility,
    WhereClause,
};

use crate::common::{expect_t, ty_bits};

pub(crate) fn bitfield_struct_impl(tokens: TokenStream) -> Result<TokenStream> {
    parse_bitfield_structs.parse2(tokens)
}

fn parse_bitfield_structs(input: ParseStream) -> Result<TokenStream> {
    let mut result = Vec::new();
    while !input.is_empty() {
        let stream = parse_bitfield_struct(input)?;
        result.push(stream);
    }

    Ok(result.into_iter().collect())
}

fn parse_bitfield_struct(input: ParseStream) -> Result<TokenStream> {
    fn get_endian_attr_val(attrs: &[&Attribute]) -> Result<Endian> {
        match attrs {
            [] => Ok(Endian::Native),
            [attr] => {
                let Meta::NameValue(MetaNameValue {
                    value: x,..
                        // Expr::Lit(ExprLit {
                        //     lit: Lit::Str(ref x),
                        //     ..
                        // }),
                }) = &attr.meta
                else {
                    return Err(Error::new_spanned(attr, "endian is not specified"));
                };
                let endian = expect_t::<LitStr>.parse2(x.to_token_stream())?.value();

                Endian::from_str(&endian).map_err(|_| {
                    Error::new_spanned(attr, "endian must be one of `little`, `big` or `nabive`")
                })
            }
            [first, .., last] => {
                let first_span = first.span();
                let last_span = last.span();
                let span = first_span.join(last_span).unwrap();
                Err(Error::new(span, "endian attribute must be 0 or 1"))
            }
        }
    }

    let ItemBitFieldStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        ..
    } = ItemBitFieldStruct::parse(input)?;

    let (endian, attrs): (Vec<_>, Vec<_>) = attrs
        .iter()
        .partition(|attr| attr.path().is_ident("endian"));

    let endian = get_endian_attr_val(&endian)?;

    let attrs: TokenStream = attrs.iter().map(ToTokens::to_token_stream).collect();
    let struct_fields: Vec<_> = fields.fields.iter().map(|v| &v.field).collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let methods = fields.gen_method(endian)?;

    Ok(quote! {
        #attrs
        #vis #struct_token #ident #ty_generics #where_clause {
            # ( #struct_fields ,)*
        }

        #[allow(non_snake_case)]
        impl #impl_generics #ident #ty_generics #where_clause {
            #methods
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Little,
    Big,
    Native,
}

impl FromStr for Endian {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "little" => Ok(Self::Little),
            "big" => Ok(Self::Big),
            "native" => Ok(Self::Native),
            _ => Err(()),
        }
    }
}

struct ItemBitFieldStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Struct,
    pub ident: Ident,
    pub generics: Generics,
    pub fields: BitFieldStructFields,
    pub _semi_token: Option<Semi>,
}

impl Parse for ItemBitFieldStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let struct_token = input.parse::<Token![struct]>()?;
        let ident = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        let where_clause = if input.lookahead1().peek(Token![where]) {
            Some(input.parse::<WhereClause>()?)
        } else {
            None
        };
        let generics = Generics {
            where_clause,
            ..generics
        };

        let content;
        braced!(content in input);

        let fields = content.parse::<BitFieldStructFields>()?;
        let _semi_token = content.parse()?;

        Ok(ItemBitFieldStruct {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            _semi_token,
        })
    }
}

struct BitFieldStructFields {
    fields: Punctuated<BitFieldStructField, Token![,]>,
}

impl BitFieldStructFields {
    fn gen_method(&self, endian: Endian) -> Result<TokenStream> {
        self.fields.iter().map(|v| v.gen_method(endian)).collect()
    }
}

impl Parse for BitFieldStructFields {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            fields: input.parse_terminated(BitFieldStructField::parse, Token![,])?,
        })
    }
}

/// flag123: u16 => {
///     #[bits(1)]
///     flag1: bool,
///     #[bits(2)]
///     flag2: u8,
///     #[bits(3)]
///     flag3: u8,
///     #[bits(10)]
///     flag4: u16,
/// }
struct BitFieldStructField {
    field: Field,
    bit_field: Option<BitField>,
}

impl Parse for BitFieldStructField {
    fn parse(input: ParseStream) -> Result<Self> {
        fn validate_bits_attributes_size(expected: u32, field: &BitFieldNamed) -> Result<()> {
            let attributes_size = field.get_bit_attrs_sum()?;
            if attributes_size != expected {
                return Err(Error::new_spanned(
                    &field.fields,
                    format!(
                        "field size does not match, expected: {}, got: {}",
                        expected, attributes_size
                    ),
                ));
            }

            Ok(())
        }

        let field = Field::parse_named(input)?;

        let bit_field = if input.lookahead1().peek(Token![=>]) {
            expect_t::<FatArrow>(input)?;
            let bit_field = input.parse::<BitField>()?;
            let ty = MyType::from_syn(&field.ty)?;
            match (ty, &bit_field) {
                (MyType::Normal(ty), BitField::Normal(normal)) => {
                    let field_size = ty.bit_size()?;
                    validate_bits_attributes_size(field_size, normal)?;
                }
                (MyType::Array(ty), BitField::Array(array)) => {
                    let expected_elem_size = ty.elem.bit_size()?;

                    for field in array.fields.iter() {
                        validate_bits_attributes_size(expected_elem_size, field)?;
                    }

                    let expected_arr_size = ty.bit_size()?;
                    let actual_arr_size = array.fields.len() as u32 * expected_elem_size;
                    if actual_arr_size != expected_arr_size {
                        return Err(Error::new_spanned(
                            field,
                            format!(
                                "array size does not match. expected: {}, got: {}",
                                expected_arr_size, actual_arr_size
                            ),
                        ));
                    }
                }

                _ => return Err(Error::new_spanned(field, "type is not match")),
            }
            Some(bit_field)
        } else {
            None
        };

        Ok(Self { field, bit_field })
    }
}

impl BitFieldStructField {
    fn gen_method(&self, endian: Endian) -> Result<TokenStream> {
        let base_field_name = self
            .field
            .ident
            .as_ref()
            .ok_or(Error::new_spanned(&self.field, "ident should not None"))?;

        let base_ty = &self.field.ty;

        let getter_ident = format_ident!("get_{base_field_name}");
        let setter_ident = format_ident!("set_{base_field_name}");
        let with_ident = format_ident!("with_{base_field_name}");
        let (getter, setter) = match endian {
            Endian::Little => (format_ident!("from_le"), format_ident!("to_le")),
            Endian::Big => (format_ident!("from_be"), format_ident!("to_be")),
            Endian::Native => (format_ident!("from"), format_ident!("into")),
        };

        let base_method = match MyType::from_syn(base_ty)? {
            MyType::Array(MyArrayType { elem_syn, .. }) => {
                quote! {
                    pub fn #getter_ident(&self) -> #base_ty {
                        self.#base_field_name.map(<#elem_syn>::#getter)
                    }

                    pub fn #setter_ident(&mut self, val: #base_ty) {
                        self.#base_field_name = val.map(<#elem_syn>::#setter);
                    }

                    pub fn #with_ident(mut self, val: #base_ty) -> Self {
                        self.#setter_ident(val);
                        self
                    }
                }
            }
            MyType::Normal(_) => {
                quote! {
                    pub fn #getter_ident(&self) -> #base_ty {
                        <#base_ty>::#getter(self.#base_field_name)
                    }

                    pub fn #setter_ident(&mut self, val: #base_ty) {
                        self.#base_field_name = val.#setter();
                    }

                    pub fn #with_ident(mut self, val: #base_ty) -> Self {
                        self.#setter_ident(val);
                        self
                    }
                }
            }
        };

        Ok(match self.bit_field {
            Some(ref bit_field) => {
                let field_method = bit_field.gen_method(base_field_name, base_ty, endian)?;
                quote! {
                    #base_method
                    #field_method
                }
            }
            None => base_method,
        })
    }
}

enum BitField {
    Normal(BitFieldNamed),
    Array(BitFieldArray),
}

impl BitField {
    fn gen_method(
        &self,
        base_field_name: &Ident,
        base_ty: &Type,
        endian: Endian,
    ) -> Result<TokenStream> {
        match self {
            BitField::Normal(normal) => {
                let accessor = quote! {self.#base_field_name};
                normal.gen_method(base_field_name, &accessor, base_ty, endian)
            }
            BitField::Array(array) => {
                let Type::Array(TypeArray { elem, .. }) = base_ty else {
                    return Err(Error::new_spanned(base_ty, "must be array type"));
                };
                array.gen_method(base_field_name, elem, endian)
            }
        }
    }
}

impl Parse for BitField {
    fn parse(input: ParseStream) -> Result<Self> {
        let field = if input.lookahead1().peek(Bracket) {
            BitField::Array(input.parse::<BitFieldArray>()?)
        } else {
            BitField::Normal(input.parse::<BitFieldNamed>()?)
        };

        Ok(field)
    }
}

/// [
///     {
///         #[bits(2)]
///         flag5: u8,
///         #[bits(2)]
///         flag6: u8,
///         #[bits(2)]
///         flag7: u8,
///         #[bits(2)]
///         flag8: u8,
///     },
///     {
///         #[bits(2)]
///         flag9: u8,
///         #[bits(2)]
///         flag10: u8,
///         #[bits(2)]
///         flag11: u8,
///         #[bits(2)]
///         flag12: u8,
///     }
/// ]
struct BitFieldArray {
    fields: Punctuated<BitFieldNamed, Token![,]>,
}

impl BitFieldArray {
    fn gen_method(
        &self,
        base_field_name: &Ident,
        base_ty: &Type,
        endian: Endian,
    ) -> Result<TokenStream> {
        let mut result = Vec::new();
        for (idx, field) in self.fields.iter().enumerate() {
            let method_prefix = format_ident!("{base_field_name}_{idx}");
            let field_accessor = quote! {self.#base_field_name[#idx]};
            let method = field.gen_method(&method_prefix, &field_accessor, base_ty, endian)?;
            result.push(method);
        }

        Ok(result.into_iter().collect())
    }
}

impl Parse for BitFieldArray {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        Ok(Self {
            fields: content.parse_terminated(BitFieldNamed::parse, Token![,])?,
        })
    }
}

/// {
///     #[bits(1)]
///     flag1: bool,
///     #[bits(2)]
///     flag2: u8,
///     #[bits(3)]
///     flag3: u8,
///     #[bits(10)]
///     flag4: u16,
/// }
struct BitFieldNamed {
    fields: FieldsNamed,
}

impl BitFieldNamed {
    fn get_bit_attrs_sum(&self) -> Result<u32> {
        let span = self.fields.span();
        self.fields
            .named
            .iter()
            .map(|field| get_bit_attr_val(&field.attrs, span))
            .sum()
    }

    fn gen_method(
        &self,
        method_prefix: &Ident,
        field_accessor: &TokenStream,
        base_ty: &Type,
        endian: Endian,
    ) -> Result<TokenStream> {
        let mut offset = 0;
        let mut methods = Vec::new();
        for field in self.fields.named.iter() {
            let (method, bit_size) = gen_field_method(
                field_accessor,
                base_ty,
                field,
                method_prefix,
                offset,
                endian,
            )?;

            methods.push(method);
            offset += bit_size;
        }

        Ok(methods.into_iter().collect())
    }
}

impl Parse for BitFieldNamed {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields = input.parse::<FieldsNamed>()?;

        Ok(Self { fields })
    }
}

impl Size for BitFieldNamed {
    fn bit_size(&self) -> Result<u32> {
        self.fields.named.iter().map(|v| ty_bits(&v.ty)).sum()
    }
}

enum MyType {
    Array(MyArrayType),
    Normal(MyNormalType),
}

impl MyType {
    fn from_syn(ty: &Type) -> Result<Self> {
        match ty {
            Type::Array(TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }),
                ..
            }) => Ok(MyType::Array(MyArrayType {
                elem: Box::new(MyType::from_syn(elem)?),
                elem_syn: *elem.clone(),
                len: lit_int.base10_parse()?,
            })),

            Type::Path(TypePath { path, .. }) => {
                let ident = path
                    .get_ident()
                    .ok_or(Error::new_spanned(ty, "single type required"))?;

                let kind = MyNormalTypeKind::from_str(ident.to_string().as_str())
                    .map_err(|_| Error::new_spanned(ident, "unsupported type"))?;
                Ok(MyType::Normal(MyNormalType {
                    ident: ident.clone(),
                    kind,
                }))
            }

            _ => Err(Error::new_spanned(ty, "type should be array or integer")),
        }
    }
}

impl Size for MyType {
    fn bit_size(&self) -> Result<u32> {
        match self {
            MyType::Array(a) => a.bit_size(),
            MyType::Normal(n) => n.bit_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MyNormalType {
    ident: Ident,
    kind: MyNormalTypeKind,
}

impl Size for MyNormalType {
    fn bit_size(&self) -> Result<u32> {
        self.kind.bit_size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MyNormalTypeKind {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    User(String),
}

impl Size for MyNormalTypeKind {
    fn bit_size(&self) -> Result<u32> {
        use MyNormalTypeKind::*;

        Ok(match self {
            Bool => 1,
            U8 | I8 => 8,
            U16 | I16 => 16,
            U32 | I32 => 32,
            U64 | I64 => 64,
            User(_) => todo!(),
        })
    }
}

impl FromStr for MyNormalTypeKind {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use MyNormalTypeKind::*;

        match s {
            "bool" => Ok(Bool),
            "u8" => Ok(U8),
            "u16" => Ok(U16),
            "u32" => Ok(U32),
            "u64" => Ok(U64),
            "i8" => Ok(I8),
            "i16" => Ok(I16),
            "i32" => Ok(I32),
            "i64" => Ok(I64),
            x => Ok(User(x.to_string())),
        }
    }
}

struct MyArrayType {
    elem: Box<MyType>,
    elem_syn: Type,
    len: u32,
}

impl Size for MyArrayType {
    fn bit_size(&self) -> Result<u32> {
        Ok(self.elem.bit_size()? * self.len)
    }
}

trait Size {
    fn bit_size(&self) -> Result<u32>;
}

fn get_bit_attr_val(attrs: &[Attribute], span: Span) -> Result<u32> {
    let bits_attr = attrs
        .iter()
        .find(|v| v.path().is_ident("bits"))
        .ok_or(Error::new(span, "bits attribute not found"))?;

    let Attribute {
        meta: Meta::List(MetaList { tokens, .. }),
        ..
    } = bits_attr
    else {
        return Err(Error::new_spanned(
            bits_attr,
            "invalid arg(s) to `bits` attribute",
        ));
    };

    let bit_size = expect_t::<LitInt>
        .parse2(tokens.clone())?
        .base10_parse::<u32>()?;

    Ok(bit_size)
}

fn gen_field_method(
    // `self.flag123` of `self.flags[0]`
    field_accessor: &TokenStream,
    field_base_ty: &Type,
    field: &Field,
    prefix: &Ident,
    offset: u32,
    endian: Endian,
) -> Result<(TokenStream, u32)> {
    let field_ident = field
        .ident
        .as_ref()
        .ok_or(Error::new_spanned(field, "ident should not be None"))?;

    let bit_size = get_bit_attr_val(&field.attrs, field.span())?;

    let set_ident = format_ident!("set_{}_{}", prefix, field_ident);
    let get_ident = format_ident!("get_{}_{}", prefix, field_ident);
    let with_ident = format_ident!("with_{}_{}", prefix, field_ident);
    let ty = &field.ty;

    let (cast_result, cast_val) = match MyType::from_syn(ty)? {
        MyType::Normal(MyNormalType {
            kind: MyNormalTypeKind::Bool,
            ..
        }) => (quote! {result != 0}, quote! {val as #field_base_ty}),
        MyType::Normal(MyNormalType {
            kind: MyNormalTypeKind::User(_),
            ..
        }) => (
            quote! {
                let result = result.wrapping_shr(#offset);
                <#ty>::from_ne(result)
            },
            quote! {val.to_ne()},
        ),
        _ => (
            quote! {result.wrapping_shr(#offset) as #ty},
            quote! {val as #field_base_ty},
        ),
    };

    let (getter_endian, setter_endian) = match endian {
        Endian::Little => (format_ident!("from_le"), format_ident!("to_le")),
        Endian::Big => (format_ident!("from_be"), format_ident!("to_be")),
        Endian::Native => (format_ident!("from"), format_ident!("into")),
    };

    Ok((
        quote! {
            pub fn #get_ident(&self) -> #ty {
                let tmp: #field_base_ty = <#field_base_ty>::#getter_endian(#field_accessor);

                // 1. まず、マスクを作成してbit_sizeの位置のビットをクリアする
                let mask: #field_base_ty = (<#field_base_ty>::wrapping_shl(1, #bit_size) - 1) << #offset;
                let result: #field_base_ty = tmp & mask;

                #cast_result
            }

            pub fn #set_ident(&mut self, val: #ty) {
                let mut tmp: #field_base_ty = #field_accessor;
                // 1. まず、マスクを作成してbit_sizeの位置のビットをクリアする
                let clear_mask: #field_base_ty = !((<#field_base_ty>::wrapping_shl(1, #bit_size) - 1) << #offset);
                tmp &= clear_mask;

                // 2. 指定されたvalueをoffset位置にシフトし、既存のデータとORを取る
                let value_mask: #field_base_ty = (#cast_val & (<#field_base_ty>::wrapping_shl(1, #bit_size) - 1)) << #offset;
                tmp |= value_mask;

                #field_accessor = tmp.#setter_endian();
            }

            pub fn #with_ident(mut self, val: #ty) -> Self {
                self.#set_ident(val);
                self
            }
        },
        bit_size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_compile_error() {
        assert!(bitfield_struct_impl(quote! {
            struct A {
                data: u8 => {
                    #[bits(1)]
                    one: bool,
                }
            }
        })
        .is_err());

        assert!(bitfield_struct_impl(quote! {
            struct A {
                data: u8 => {
                    #[bits(2)]
                    one: bool,
                }
            }
        })
        .is_err());

        assert!(bitfield_struct_impl(quote! {
            struct A {
                data: [u8; 2] => [
                    {
                        #[bits(8)]
                        one: u8,
                    },
                ]
            }
        })
        .is_err());

        assert!(bitfield_struct_impl(quote! {
            #[endian = ""]
            struct A {
                data: u8,
            }
        })
        .is_err());
    }

    #[test]
    fn test_bitfield() {
        assert_eq!(
            bitfield_struct_impl(quote! {
                struct A {}
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                struct A {}
                #[allow(non_snake_case)]
                impl A {}
            }
            .to_string()
        );

        assert_eq!(
            bitfield_struct_impl(quote! {
                struct A<T> where T: Debug + Default {}
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                struct A<T> where T: Debug + Default {}
                #[allow(non_snake_case)]
                impl<T> A<T>
                where
                    T: Debug + Default
                {}
            }
            .to_string()
        );

        assert_eq!(
            bitfield_struct_impl(quote! {
                struct A {
                    hoge: u16 => {
                        #[bits(16)]
                        ptr: u16
                    },
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                struct A {
                    hoge: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_hoge(&self) -> u16 {
                        <u16>::from(self.hoge)
                    }
                    pub fn set_hoge(&mut self, val: u16) {
                        self.hoge = val.into();
                    }
                    pub fn with_hoge(mut self, val: u16) -> Self {
                        self.set_hoge(val);
                        self
                    }
                    pub fn get_hoge_ptr(&self) -> u16 {
                        let tmp: u16 = <u16>::from(self.hoge);
                        let mask: u16 = (<u16>::wrapping_shl(1, 16u32) - 1)<< 0u32;
                        let result: u16 = tmp & mask;
                        result.wrapping_shr(0u32) as u16
                    }
                    pub fn set_hoge_ptr(&mut self, val: u16) {
                        let mut tmp: u16 = self.hoge;
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 16u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & (<u16>::wrapping_shl(1, 16u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.hoge = tmp.into();
                    }
                    pub fn with_hoge_ptr(mut self, val: u16) -> Self {
                        self.set_hoge_ptr(val);
                        self
                    }
                }
            }
            .to_string()
        );

        assert_eq!(
            bitfield_struct_impl(quote! {
                struct A {
                    hoge: u16 => {
                        #[bits(8)]
                        flag: Flag,
                        #[bits(8)]
                        flag2: Flag,
                    },
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                struct A {
                    hoge: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_hoge(&self) -> u16 {
                        <u16>::from(self.hoge)
                    }
                    pub fn set_hoge(&mut self, val: u16) {
                        self.hoge = val.into();
                    }
                    pub fn with_hoge(mut self, val: u16) -> Self {
                        self.set_hoge(val);
                        self
                    }
                    pub fn get_hoge_flag(&self) -> Flag {
                        let tmp: u16 = <u16>::from(self.hoge);
                        let mask: u16 = (<u16>::wrapping_shl(1, 8u32) - 1)<< 0u32;
                        let result: u16 = tmp & mask;
                        let result = result.wrapping_shr(0u32);
                        <Flag>::from_ne(result)
                    }
                    pub fn set_hoge_flag(&mut self, val: Flag) {
                        let mut tmp: u16 = self.hoge;
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 8u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val.to_ne() & (<u16>::wrapping_shl(1, 8u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.hoge = tmp.into();
                    }
                    pub fn with_hoge_flag(mut self, val: Flag) -> Self {
                        self.set_hoge_flag(val);
                        self
                    }
                    pub fn get_hoge_flag2(&self) -> Flag {
                        let tmp: u16 = <u16>::from(self.hoge);
                        let mask: u16 = (<u16>::wrapping_shl(1, 8u32) - 1)<< 8u32;
                        let result: u16 = tmp & mask;
                        let result = result.wrapping_shr(8u32);
                        <Flag>::from_ne(result)
                    }
                    pub fn set_hoge_flag2(&mut self, val: Flag) {
                        let mut tmp: u16 = self.hoge;
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 8u32) - 1) << 8u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val.to_ne() & (<u16>::wrapping_shl(1, 8u32) - 1)) << 8u32;
                        tmp |= value_mask;
                        self.hoge = tmp.into();
                    }
                    pub fn with_hoge_flag2(mut self, val: Flag) -> Self {
                        self.set_hoge_flag2(val);
                        self
                    }
                }
            }
            .to_string()
        );

        // no endian: native
        assert_eq!(
            bitfield_struct_impl(quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> u16 {
                        <u16>::from(self.flag123)
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val.into();
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                }
            }
            .to_string()
        );

        // native
        assert_eq!(
            bitfield_struct_impl(quote! {
                #[derive(Debug)]
                #[repr(C)]
                #[endian = "native"]
                pub struct A {
                    flag123: u16,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> u16 {
                        <u16>::from(self.flag123)
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val.into();
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                }
            }
            .to_string()
        );

        // little
        assert_eq!(
            bitfield_struct_impl(quote! {
                #[derive(Debug)]
                #[endian = "little"]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> u16 {
                        <u16>::from_le(self.flag123)
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val.to_le();
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                }
            }
            .to_string()
        );

        // big
        assert_eq!(
            bitfield_struct_impl(quote! {
                #[endian = "big"]
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: u16,
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> u16 {
                        <u16>::from_be(self.flag123)
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val.to_be();
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                }
            }
            .to_string()
        );

        assert_eq!(
            bitfield_struct_impl(quote! {
                #[endian = "big"]
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: [u16; 1] => [
                        {
                            #[bits(16)]
                            flag: u16,
                        }
                    ],
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                #[repr(C)]
                pub struct A {
                    flag123: [u16; 1],
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> [u16; 1] {
                        self.flag123.map(<u16>::from_be)
                    }
                    pub fn set_flag123(&mut self, val: [u16; 1]) {
                        self.flag123 = val.map(<u16>::to_be);
                    }
                    pub fn with_flag123(mut self, val: [u16; 1]) -> Self {
                        self.set_flag123(val);
                        self
                    }
                    pub fn get_flag123_0_flag(&self) -> u16 {
                        let tmp: u16 = <u16>::from_be(self.flag123[0usize]);
                        let mask: u16 = (<u16>::wrapping_shl(1, 16u32) - 1)<< 0u32;
                        let result: u16 = tmp & mask;
                        result.wrapping_shr(0u32) as u16
                    }
                    pub fn set_flag123_0_flag(&mut self, val: u16) {
                        let mut tmp: u16 = self.flag123[0usize];
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 16u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & (<u16>::wrapping_shl(1, 16u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flag123[0usize] = tmp.to_be();
                    }
                    pub fn with_flag123_0_flag(mut self, val: u16) -> Self {
                        self.set_flag123_0_flag(val);
                        self
                    }
                }
            }
            .to_string()
        );

        assert_eq!(
            bitfield_struct_impl(quote! {
                #[derive(Debug)]
                pub struct A {
                    flag123: u16 => {
                       #[bits(1)]
                       flag1: bool,
                       #[bits(15)]
                       flag2: u16,
                    },
                    flags: [u8; 2] => [
                        {
                            #[bits(8)]
                            flag3: u8,
                        },
                        {
                            #[bits(2)]
                            flag4: u8,
                            #[bits(6)]
                            flag5: u8,
                        }
                    ]
                }
            })
            .unwrap_or_else(Error::into_compile_error)
            .to_string(),
            quote! {
                #[derive(Debug)]
                pub struct A {
                    flag123: u16,
                    flags: [u8; 2],
                }
                #[allow(non_snake_case)]
                impl A {
                    pub fn get_flag123(&self) -> u16 {
                        <u16>::from(self.flag123)
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val.into();
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                    pub fn get_flag123_flag1(&self) -> bool {
                        let tmp: u16 = <u16>::from(self.flag123);
                        let mask: u16 = (<u16>::wrapping_shl(1, 1u32) - 1)<< 0u32;
                        let result: u16 = tmp & mask;
                        result != 0
                    }
                    pub fn set_flag123_flag1(&mut self, val: bool) {
                        let mut tmp: u16 = self.flag123;
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 1u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & (<u16>::wrapping_shl(1, 1u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flag123 = tmp.into();
                    }
                    pub fn with_flag123_flag1(mut self, val: bool) -> Self {
                        self.set_flag123_flag1(val);
                        self
                    }
                    pub fn get_flag123_flag2(&self) -> u16 {
                        let tmp: u16 = <u16>::from(self.flag123);
                        let mask: u16 = (<u16>::wrapping_shl(1, 15u32) - 1)<< 1u32;
                        let result: u16 = tmp & mask;
                        result.wrapping_shr(1u32) as u16
                    }
                    pub fn set_flag123_flag2(&mut self, val: u16) {
                        let mut tmp: u16 = self.flag123;
                        let clear_mask: u16 = !((<u16>::wrapping_shl(1, 15u32) - 1) << 1u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & (<u16>::wrapping_shl(1, 15u32) - 1)) << 1u32;
                        tmp |= value_mask;
                        self.flag123 = tmp.into();
                    }
                    pub fn with_flag123_flag2(mut self, val: u16) -> Self {
                        self.set_flag123_flag2(val);
                        self
                    }
                    pub fn get_flags(&self) -> [u8; 2] {
                        self.flags.map(<u8>::from)
                    }
                    pub fn set_flags(&mut self, val: [u8; 2]) {
                        self.flags = val.map(<u8>::into);
                    }
                    pub fn with_flags(mut self, val: [u8; 2]) -> Self {
                        self.set_flags(val);
                        self
                    }
                    pub fn get_flags_0_flag3(&self) -> u8 {
                        let tmp: u8 = <u8>::from(self.flags[0usize]);
                        let mask: u8 = (<u8>::wrapping_shl(1, 8u32) - 1)<< 0u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(0u32) as u8
                    }
                    pub fn set_flags_0_flag3(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[0usize];
                        let clear_mask: u8 = !((<u8>::wrapping_shl(1, 8u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & (<u8>::wrapping_shl(1, 8u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flags[0usize] = tmp.into();
                    }
                    pub fn with_flags_0_flag3(mut self, val: u8) -> Self {
                        self.set_flags_0_flag3(val);
                        self
                    }
                    pub fn get_flags_1_flag4(&self) -> u8 {
                        let tmp: u8 = <u8>::from(self.flags[1usize]);
                        let mask: u8 = (<u8>::wrapping_shl(1, 2u32) - 1)<< 0u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(0u32) as u8
                    }
                    pub fn set_flags_1_flag4(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[1usize];
                        let clear_mask: u8 = !((<u8>::wrapping_shl(1, 2u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & (<u8>::wrapping_shl(1, 2u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flags[1usize] = tmp.into();
                    }
                    pub fn with_flags_1_flag4(mut self, val: u8) -> Self {
                        self.set_flags_1_flag4(val);
                        self
                    }
                    pub fn get_flags_1_flag5(&self) -> u8 {
                        let tmp: u8 = <u8>::from(self.flags[1usize]);
                        let mask: u8 = (<u8>::wrapping_shl(1, 6u32) - 1)<< 2u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(2u32) as u8
                    }
                    pub fn set_flags_1_flag5(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[1usize];
                        let clear_mask: u8 = !((<u8>::wrapping_shl(1, 6u32) - 1) << 2u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & (<u8>::wrapping_shl(1, 6u32) - 1)) << 2u32;
                        tmp |= value_mask;
                        self.flags[1usize] = tmp.into();
                    }
                    pub fn with_flags_1_flag5(mut self, val: u8) -> Self {
                        self.set_flags_1_flag5(val);
                        self
                    }
                }
            }
            .to_string()
        );
    }
}
