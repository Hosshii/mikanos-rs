use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
// `remove unused imports` token::Bracket regardless it is used or not.
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    token::{Bracket, FatArrow, Semi, Struct},
    Attribute, Error, Field, FieldsNamed, Generics, Ident, LitInt, Meta, MetaList, Result, Token,
    Type, TypeArray, Visibility, WhereClause,
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
        let field = Field::parse_named(input)?;

        let bit_field = if input.lookahead1().peek(Token![=>]) {
            expect_t::<FatArrow>(input)?;
            let bit_field = input.parse::<BitField>()?;
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

        if offset != ty_bits(base_ty)? {
            return Err(Error::new_spanned(
                &self.fields,
                format!(
                    "sum of bits does not match. expected: {}, got: {}",
                    ty_bits(base_ty)?,
                    offset
                ),
            ));
        }

        Ok(methods.into_iter().collect())
    }
}

impl Parse for BitFieldNamed {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            fields: input.parse()?,
        })
    }
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

    let bits_attr = field
        .attrs
        .iter()
        .find(|v| v.path().is_ident("bits"))
        .ok_or(Error::new_spanned(field, "bits attribute not found"))?;
    let Attribute {
        meta: Meta::List(MetaList { tokens, .. }),
        ..
    } = bits_attr
    else {
        return Err(Error::new_spanned(bits_attr, "invalid arg"));
    };

    let set_ident = format_ident!("set_{}_{}", prefix, field_ident);
    let get_ident = format_ident!("get_{}_{}", prefix, field_ident);
    let with_ident = format_ident!("with_{}_{}", prefix, field_ident);
    let ty = &field.ty;
    let ty_bits_size = ty_bits(ty)?;
    let bit_size = expect_t::<LitInt>
        .parse2(tokens.clone())?
        .base10_parse::<u32>()?;

    if ty_bits_size < bit_size {
        return Err(Error::new_spanned(
            field,
            format!(
                "field size {} is smaller than bits attribute {}",
                ty_bits_size, bit_size
            ),
        ));
    }

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
                struct A<T, U> where T: Debug + Default {}
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
