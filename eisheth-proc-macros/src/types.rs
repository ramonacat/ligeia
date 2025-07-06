use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn rust_type_to_eisheth_type_instance(r#type: &Type, with_super: bool) -> TokenStream {
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
                ident_to_type_instance(ident, with_super)
            } else {
                todo!("path multiple idents");
            }
        }
        Type::Ptr(target) => {
            let r#mut = target.mutability;
            let r#const = target.const_token;

            // TODO: This is hacky and will break if we introduce pointer types that carry value
            // types
            quote! { <* #r#mut #r#const u8 as ::eisheth::types::RepresentedAs>::representation() }
        }
        Type::Reference(_) => todo!("Reference"),
        Type::Slice(_) => todo!("Slice"),
        Type::TraitObject(_) => todo!("TraitObject"),
        Type::Tuple(_) => todo!("Tuple"),
        Type::Verbatim(_) => todo!("Verbatim"),
        _ => todo!(),
    }
}

pub fn ident_to_type_instance(ident: &Ident, with_super: bool) -> proc_macro2::TokenStream {
    let qualifier = if with_super
        && (
            // TODO can we check if the ident is for a builtin type somehow, instead of this series of
            // comparisons?
            ident != "u8" && ident != "u16" && ident != "u32" && ident != "u64"
        ) {
        Some(quote! {super::})
    } else {
        None
    };

    quote! { < #qualifier #ident as ::eisheth::types::RepresentedAs >::representation() }
}
