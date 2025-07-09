use proc_macro::TokenStream;

use crate::items::{
    enums::ffi_enum_inner, modules::define_module_inner, structs::ffi_struct_inner,
};

mod convert_case;
mod items;
mod types;

#[proc_macro_attribute]
pub fn ffi_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    ffi_struct_inner(attr, item)
}

#[proc_macro_attribute]
pub fn ffi_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    ffi_enum_inner(attr, item)
}

#[proc_macro]
pub fn define_module(tokens: TokenStream) -> TokenStream {
    define_module_inner(tokens)
}
