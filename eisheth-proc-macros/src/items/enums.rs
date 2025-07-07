use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, parse_macro_input};

pub fn ffi_enum_inner(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemEnum);

    let mut repr_value = None;
    for attr in &item.attrs {
        if attr.path().is_ident("repr") {
            let received_repr_value = attr.parse_args::<Ident>().unwrap();

            if ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"]
                .contains(&(received_repr_value.to_string().as_str()))
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

        impl ::eisheth::types::RepresentedAs for #name {
            type RepresentationType = <#repr_value as ::eisheth::types::RepresentedAs>::RepresentationType;

            fn representation() -> <#repr_value as ::eisheth::types::RepresentedAs>::RepresentationType {
                <#repr_value as ::eisheth::types::RepresentedAs>::representation()
            }
        }
    }
    .into()
}
