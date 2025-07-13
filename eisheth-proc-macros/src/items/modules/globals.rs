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
    let rust_type = &global_declaration.r#type;
    let r#type = rust_type_to_eisheth_type_instance(rust_type);
    let visibility = match visibility {
        grammar::Visibility::Export => quote! { Export },
        grammar::Visibility::Internal => quote! { Internal },
    };

    let value = global_declaration.value.as_ref().map_or_else(
        || quote! { None },
        |(_, value)| quote! { Some(&(#value as #rust_type).into()) },
    );

    quote! {
        // TODO support setting a const initializer
        let #name = module.define_global(
            ::eisheth::Visibility::#visibility,
            #name_str,
            #r#type,
            #value
        );
    }
}

pub fn make_global_getter(declaration: &grammar::GlobalDeclaration) -> proc_macro2::TokenStream {
    let name = &declaration.name;

    let getter_name = format_ident!("get_{}", name);

    quote! {
        pub fn #getter_name<'module>(
            &self,
        ) -> ::eisheth::module::DeclaredGlobalDescriptor {
            self.#name
        }
    }
}
