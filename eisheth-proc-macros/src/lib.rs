use proc_macro::TokenStream;

use crate::items::{
    enums::ffi_enum_inner,
    function_signature::{define_module_function_caller_inner, define_module_function_inner},
    structs::ffi_struct_inner,
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
pub fn define_module_function_caller(tokens: TokenStream) -> TokenStream {
    define_module_function_caller_inner(tokens)
}

#[proc_macro]
pub fn define_module_function(tokens: TokenStream) -> TokenStream {
    define_module_function_inner(tokens)
}
