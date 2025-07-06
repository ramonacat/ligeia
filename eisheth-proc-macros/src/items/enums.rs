use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, parse_macro_input};

pub fn ffi_enum_inner(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemEnum);

    let mut repr_value = None;
    for attr in &item.attrs {
        if attr.path().is_ident("repr") {
            let received_repr_value = attr.parse_args::<Ident>().unwrap();

            // TODO signed ok too?
            if received_repr_value == "u8"
                || received_repr_value == "u16"
                || received_repr_value == "u32"
                || received_repr_value == "u64"
            {
                repr_value = Some(received_repr_value);
            }
        }
    }

    let repr_value = repr_value
        .expect("To be FFI-compatible, the enum must have a #[repr()] that is an integer type");

    let name = &item.ident;

    quote! {
        #item

        // TODO can we blanket implement Type for all RepresentedAs?
        impl ::eisheth::types::Type for #name {
            fn as_llvm_ref(&self) -> ::eisheth::llvm_sys::prelude::LLVMTypeRef {
                <Self as ::eisheth::types::RepresentedAs>::representation().as_llvm_ref()
            }
        }

        impl ::eisheth::types::RepresentedAs for #name {
            type RepresentationType = <#repr_value as ::eisheth::types::RepresentedAs>::RepresentationType;

            fn representation() -> <#repr_value as ::eisheth::types::RepresentedAs>::RepresentationType {
                <#repr_value as ::eisheth::types::RepresentedAs>::representation()
            }
        }
    }
    .into()
}
