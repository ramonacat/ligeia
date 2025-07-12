use quote::{format_ident, quote};

use crate::{
    items::modules::grammar::{self, Visibility},
    types::rust_type_to_eisheth_type_instance,
};

pub fn make_global_declaration(
    visibility: Visibility,
    global_declaration: &grammar::GlobalDeclaration,
) -> proc_macro2::TokenStream {
    let name = &global_declaration.name;
    let name_str = name.to_string();
    let r#type = rust_type_to_eisheth_type_instance(&global_declaration.r#type);
    let visibility = match visibility {
        grammar::Visibility::Export => quote! { Export },
        grammar::Visibility::Internal => quote! { Internal },
    };

    quote! {
        // TODO support setting a const initializer
        let #name = module.define_global(::eisheth::function::declaration::Visibility::#visibility, #name_str, #r#type, None);
    }
}

pub fn make_global_getter(declaration: &grammar::GlobalDeclaration) -> proc_macro2::TokenStream {
    let name = &declaration.name;

    let getter_name = format_ident!("get_{}", name);

    quote! {
        pub fn #getter_name<'module>(
            &self,
            i: &'module ::eisheth::function::instruction_builder::InstructionBuilder<'module>
        ) -> ::eisheth::module::GlobalReference<'module> {
            i.module().get_global(self.#name)
        }
    }
}
