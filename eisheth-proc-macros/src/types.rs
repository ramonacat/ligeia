use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn rust_to_eisheth_type(r#type: &Type, with_super: bool) -> TokenStream {
    match r#type {
        Type::Array(_) => todo!("Array"),
        Type::BareFn(_) => todo!("BareFn"),
        Type::Group(_) => todo!("Group"),
        Type::ImplTrait(_) => todo!("ImplTrait"),
        Type::Infer(_) => todo!("Infer"),
        Type::Macro(_) => todo!("Macro"),
        Type::Never(_) => todo!("Never"),
        Type::Paren(_) => todo!("Paren"),
        Type::Path(path) => {
            if path.qself.is_some() {
                todo!("path.qself");
            } else if let Some(ident) = path.path.get_ident() {
                ident_to_type(ident, with_super)
            } else {
                todo!("path multiple idents");
            }
        }
        Type::Ptr(_) => {
            quote! { ::eisheth::types::Pointer }
        }
        Type::Reference(_) => todo!("Reference"),
        Type::Slice(_) => todo!("Slice"),
        Type::TraitObject(_) => todo!("TraitObject"),
        Type::Tuple(_) => todo!("Tuple"),
        Type::Verbatim(_) => todo!("Verbatim"),
        _ => todo!(),
    }
}

pub fn ident_to_type(ident: &Ident, with_super: bool) -> proc_macro2::TokenStream {
    match ident.to_string().as_str() {
        "u8" => quote! { ::eisheth::types::U8 },
        "u16" => quote! { ::eisheth::types::U16 },
        "u32" => quote! { ::eisheth::types::U32 },
        "u64" => quote! { ::eisheth::types::U64 },
        _ => {
            let qualifier = if with_super {
                Some(quote! {super::})
            } else {
                None
            };
            // TODO can we just do this for all the types?
            quote! { < #qualifier #ident as ::eisheth::types::RepresentedAs >::REPRESENTATION }
        }
    }
}
