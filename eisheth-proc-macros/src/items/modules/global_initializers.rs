use quote::quote;

use crate::items::modules::grammar::GlobalInitializerDeclaration;

pub fn make_global_initializer(gid: &GlobalInitializerDeclaration) -> proc_macro2::TokenStream {
    let priority = &gid.priority;

    let initialized_data_pointer = gid.data_pointer.as_ref().map_or_else(
        || quote! { None },
        |(_, name)| quote! { Some(module.get_global(#name).into()) },
    );

    let initializer = &gid.initializer_fn;

    quote! {
        module.define_global_initializer(
            #priority,
            #initialized_data_pointer,
            #initializer,
        );
    }
}
