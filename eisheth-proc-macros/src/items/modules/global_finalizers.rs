use quote::quote;

use crate::items::modules::grammar::GlobalFinalizerDeclaration;

pub fn make_global_finalizer(gdf: &GlobalFinalizerDeclaration) -> proc_macro2::TokenStream {
    let priority = &gdf.priority;

    let finalized_data_pointer = gdf.data_pointer.as_ref().map_or_else(
        || quote! { None },
        |(_, name)| quote! { Some(module.get_global(#name).into()) },
    );

    let finalizer = &gdf.finalizer_fn;

    quote! {
        module.define_global_finalizer(
            #priority,
            #finalized_data_pointer,
            #finalizer,
        );
    }
}
