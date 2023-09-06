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
    Attribute, Error, Expr, ExprLit, Field, FieldMutability, FieldsNamed, Generics, Ident, Lit,
    LitInt, Meta, MetaList, Result, Token, Type, TypeArray, TypePath, Visibility, WhereClause,
};

trait Validate {
    fn validate(&self) -> Result<()>;
}

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
    let ItemBitFieldStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        ..
    } = ItemBitFieldStruct::parse(input)?;
    let attrs: TokenStream = attrs.iter().map(ToTokens::to_token_stream).collect();
    let struct_fields: Vec<_> = fields.fields.iter().map(|v| &v.field).collect();
    let struct_fields_init: TokenStream = struct_fields
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let ty = &v.ty;
            quote! {
                #ident: <#ty>::default(),
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let methods = fields.gen_method()?;

    Ok(quote! {
        #attrs
        #vis #struct_token #ident #ty_generics #where_clause {
            # ( #struct_fields ,)*
        }

        #[allow(non_snake_case)]
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn new() -> Self {
                Self{
                    #struct_fields_init
                }
            }
            #methods
        }

        impl #impl_generics Default for #ident #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }
    })
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
    fn gen_method(&self) -> Result<TokenStream> {
        self.fields
            .iter()
            .map(BitFieldStructField::gen_method)
            .collect()
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
    fn gen_method(&self) -> Result<TokenStream> {
        let base_field_name = self
            .field
            .ident
            .as_ref()
            .ok_or(Error::new_spanned(&self.field, "ident should not None"))?;

        let base_ty = &self.field.ty;

        let getter_ident = format_ident!("get_{base_field_name}");
        let setter_ident = format_ident!("set_{base_field_name}");
        let with_ident = format_ident!("with_{base_field_name}");
        let base_method = quote! {
            pub fn #getter_ident(&self) -> #base_ty {
                self.#base_field_name
            }

            pub fn #setter_ident(&mut self, val: #base_ty) {
                self.#base_field_name = val;
            }

            pub fn #with_ident(mut self, val: #base_ty) -> Self {
                self.#setter_ident(val);
                self
            }
        };

        Ok(match self.bit_field {
            Some(ref bit_field) => {
                let field_method = bit_field.gen_method(base_field_name, base_ty)?;
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
    fn gen_method(&self, base_field_name: &Ident, base_ty: &Type) -> Result<TokenStream> {
        match self {
            BitField::Normal(normal) => {
                let accessor = quote! {self.#base_field_name};
                normal.gen_method(base_field_name, &accessor, base_ty)
            }
            BitField::Array(array) => {
                let Type::Array(TypeArray { elem, .. }) = base_ty else {
                    return Err(Error::new_spanned(base_ty, "must be array type"));
                };
                array.gen_method(base_field_name, elem)
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
    fn gen_method(&self, base_field_name: &Ident, base_ty: &Type) -> Result<TokenStream> {
        let mut result = Vec::new();
        for (idx, field) in self.fields.iter().enumerate() {
            let method_prefix = format_ident!("{base_field_name}_{idx}");
            let field_accessor = quote! {self.#base_field_name[#idx]};
            let method = field.gen_method(&method_prefix, &field_accessor, base_ty)?;
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
    ) -> Result<TokenStream> {
        let mut offset = 0;
        let mut methods = Vec::new();
        for field in self.fields.named.iter() {
            let (method, bit_size) =
                gen_field_method(field_accessor, base_ty, field, method_prefix, offset)?;

            methods.push(method);
            offset += bit_size;
        }

        Ok(methods.into_iter().collect())
    }
}

impl Parse for BitFieldNamed {
    fn parse(input: ParseStream) -> Result<Self> {
        let fields = input.parse::<FieldsNamed>()?;

        for field in fields.named.iter() {
            let attrs_bit_size = get_bit_attr_val(&field.attrs, field.span())?;
            let actual_bit_size = ty_bits(&field.ty)?;
            if actual_bit_size < attrs_bit_size {
                return Err(Error::new_spanned(
                    field,
                    format!(
                        "field size {} is smaller than bits attribute {}",
                        actual_bit_size, attrs_bit_size
                    ),
                ));
            }
        }

        Ok(Self { fields })
    }
}

impl Size for BitFieldNamed {
    fn bit_size(&self) -> Result<u32> {
        self.fields.named.iter().map(|v| ty_bits(&v.ty)).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MyArrayType {
    elem: Box<MyType>,
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

    let cast_result = if ty_bits(ty)? == 1 {
        quote! {result != 0}
    } else {
        quote! {result.wrapping_shr(#offset) as #ty}
    };

    Ok((
        quote! {
            pub fn #get_ident(&self) -> #ty {
                let tmp: #field_base_ty = #field_accessor;

                // 1. まず、マスクを作成してbit_sizeの位置のビットをクリアする
                let mask: #field_base_ty = ((1 << #bit_size) - 1) << #offset;
                let result: #field_base_ty = tmp & mask;

                #cast_result
            }

            pub fn #set_ident(&mut self, val: #ty) {
                let mut tmp: #field_base_ty = #field_accessor;
                // 1. まず、マスクを作成してbit_sizeの位置のビットをクリアする
                let clear_mask: #field_base_ty = !(((1 << #bit_size) - 1) << #offset);
                tmp &= clear_mask;

                // 2. 指定されたvalueをoffset位置にシフトし、既存のデータとORを取る
                let value_mask: #field_base_ty = (val as #field_base_ty & ((1 << #bit_size) - 1)) << #offset;
                tmp |= value_mask;

                #field_accessor = tmp;
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
                impl A {
                    pub fn new() -> Self {
                        Self {}
                    }
                }

                impl Default for A {
                    fn default() -> Self {
                        Self::new()
                    }
                }
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
                {
                    pub fn new() -> Self {
                        Self {}
                    }
                }

                impl<T> Default for A<T>
                where
                    T: Debug + Default
                {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
            .to_string()
        );

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
                    pub fn new() -> Self {
                        Self {
                            flag123: <u16>::default(),
                        }
                    }
                    pub fn get_flag123(&self) -> u16 {
                        self.flag123
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val;
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                }

                impl Default for A {
                    fn default() -> Self {
                        Self::new()
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
                    pub fn new() -> Self {
                        Self {
                            flag123: <u16>::default(),
                            flags: <[u8; 2]>::default(),
                        }
                    }
                    pub fn get_flag123(&self) -> u16 {
                        self.flag123
                    }
                    pub fn set_flag123(&mut self, val: u16) {
                        self.flag123 = val;
                    }
                    pub fn with_flag123(mut self, val: u16) -> Self {
                        self.set_flag123(val);
                        self
                    }
                    pub fn get_flag123_flag1(&self) -> bool {
                        let tmp: u16 = self.flag123;
                        let mask: u16 = ((1 << 1u32) - 1) << 0u32;
                        let result: u16 = tmp & mask;
                        result != 0
                    }
                    pub fn set_flag123_flag1(&mut self, val: bool) {
                        let mut tmp: u16 = self.flag123;
                        let clear_mask: u16 = !(((1 << 1u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & ((1 << 1u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flag123 = tmp;
                    }
                    pub fn with_flag123_flag1(mut self, val: bool) -> Self {
                        self.set_flag123_flag1(val);
                        self
                    }
                    pub fn get_flag123_flag2(&self) -> u16 {
                        let tmp: u16 = self.flag123;
                        let mask: u16 = ((1 << 15u32) - 1) << 1u32;
                        let result: u16 = tmp & mask;
                        result.wrapping_shr(1u32) as u16
                    }
                    pub fn set_flag123_flag2(&mut self, val: u16) {
                        let mut tmp: u16 = self.flag123;
                        let clear_mask: u16 = !(((1 << 15u32) - 1) << 1u32);
                        tmp &= clear_mask;
                        let value_mask: u16 = (val as u16 & ((1 << 15u32) - 1)) << 1u32;
                        tmp |= value_mask;
                        self.flag123 = tmp;
                    }
                    pub fn with_flag123_flag2(mut self, val: u16) -> Self {
                        self.set_flag123_flag2(val);
                        self
                    }
                    pub fn get_flags(&self) -> [u8; 2] {
                        self.flags
                    }
                    pub fn set_flags(&mut self, val: [u8; 2]) {
                        self.flags = val;
                    }
                    pub fn with_flags(mut self, val: [u8; 2]) -> Self {
                        self.set_flags(val);
                        self
                    }
                    pub fn get_flags_0_flag3(&self) -> u8 {
                        let tmp: u8 = self.flags[0usize];
                        let mask: u8 = ((1 << 8u32) - 1) << 0u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(0u32) as u8
                    }
                    pub fn set_flags_0_flag3(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[0usize];
                        let clear_mask: u8 = !(((1 << 8u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & ((1 << 8u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flags[0usize] = tmp;
                    }
                    pub fn with_flags_0_flag3(mut self, val: u8) -> Self {
                        self.set_flags_0_flag3(val);
                        self
                    }
                    pub fn get_flags_1_flag4(&self) -> u8 {
                        let tmp: u8 = self.flags[1usize];
                        let mask: u8 = ((1 << 2u32) - 1) << 0u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(0u32) as u8
                    }
                    pub fn set_flags_1_flag4(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[1usize];
                        let clear_mask: u8 = !(((1 << 2u32) - 1) << 0u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & ((1 << 2u32) - 1)) << 0u32;
                        tmp |= value_mask;
                        self.flags[1usize] = tmp;
                    }
                    pub fn with_flags_1_flag4(mut self, val: u8) -> Self {
                        self.set_flags_1_flag4(val);
                        self
                    }
                    pub fn get_flags_1_flag5(&self) -> u8 {
                        let tmp: u8 = self.flags[1usize];
                        let mask: u8 = ((1 << 6u32) - 1) << 2u32;
                        let result: u8 = tmp & mask;
                        result.wrapping_shr(2u32) as u8
                    }
                    pub fn set_flags_1_flag5(&mut self, val: u8) {
                        let mut tmp: u8 = self.flags[1usize];
                        let clear_mask: u8 = !(((1 << 6u32) - 1) << 2u32);
                        tmp &= clear_mask;
                        let value_mask: u8 = (val as u8 & ((1 << 6u32) - 1)) << 2u32;
                        tmp |= value_mask;
                        self.flags[1usize] = tmp;
                    }
                    pub fn with_flags_1_flag5(mut self, val: u8) -> Self {
                        self.set_flags_1_flag5(val);
                        self
                    }
                }
                impl Default for A {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
            .to_string()
        );
    }
}
