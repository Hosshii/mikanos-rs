use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, ExprLit, Lit, Result, Type, TypeArray, TypePath,
};

pub(crate) fn expect_t<T: Parse>(input: ParseStream) -> Result<T> {
    match input.parse::<T>() {
        Ok(t) => Ok(t),
        Err(e) => Err(Error::new(
            input.span(),
            format!("expected: {}\nerror: {}", std::any::type_name::<T>(), e),
        )),
    }
}

pub(crate) fn ty_bits(ty: &Type) -> Result<u32> {
    match ty {
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
            let elem_bits = ty_bits(elem)?;

            Ok(len * elem_bits)
        }

        Type::Path(TypePath { path, .. }) => {
            let ident = path
                .get_ident()
                .ok_or(Error::new_spanned(ty, "single type required"))?;

            match ident.to_string().as_str() {
                "bool" => Ok(1),
                "u8" | "i8" => Ok(8),
                "u16" | "i16" => Ok(16),
                "u32" | "i32" => Ok(32),
                "u64" | "i64" => Ok(64),

                x => Err(Error::new_spanned(
                    ident,
                    format!("{x} is unsupported type"),
                )),
            }
        }

        _ => Err(Error::new_spanned(ty, "type should be array or integer")),
    }
}
