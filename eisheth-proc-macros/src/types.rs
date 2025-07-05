use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub fn rust_to_eisheth_type(r#type: &Type) -> TokenStream {
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
                match ident.to_string().as_str() {
                    "u32" => quote! { ::eisheth::types::U32 },
                    id => todo!("path ident: {id}"),
                }
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
