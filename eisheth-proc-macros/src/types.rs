use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub fn rust_type_to_eisheth_type_instance(r#type: &Type) -> TokenStream {
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
                quote! { < #ident as ::eisheth::types::RepresentedAs >::representation() }
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
