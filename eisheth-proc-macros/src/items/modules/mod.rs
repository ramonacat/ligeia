use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, parse_macro_input};

use crate::items::modules::{
    functions::{make_module_function_caller, make_module_function_definition},
    global_finalizers::make_global_finalizer,
    global_initializers::make_global_initializer,
    globals::{make_global_declaration, make_global_getter},
    grammar::{DefineModuleInput, Item},
};

mod functions;
mod global_finalizers;
mod global_initializers;
mod globals;
mod grammar;

fn make_definition_struct<'a>(
    items: impl Iterator<Item = &'a Item> + Clone,
) -> proc_macro2::TokenStream {
    let definition_fields = items.clone().filter_map(|x| match &x.kind {
        grammar::ItemKind::Function(f) => {
            let name = &f.name;

            Some(quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor })
        }
        grammar::ItemKind::Global(g) => {
            let name = &g.name;

            Some(quote! { #name: ::eisheth::module::DeclaredGlobalDescriptor })
        }
        grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => None,
    });

    let item_imports = items
        .clone()
        .filter(|x| x.is_exported())
        .filter_map(|x| match &x.kind {
            grammar::ItemKind::Function(f) => {
                let name = &f.name;

                Some(quote! { let #name = module.import_function(self.#name).unwrap(); })
            }
            grammar::ItemKind::Global(g) => {
                let name = &g.name;

                Some(quote! { let #name = module.import_global(self.#name); })
            }
            grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => None,
        });

    let imported_item_names = items
        .filter(|x| x.is_exported())
        .filter_map(|x| match &x.kind {
            grammar::ItemKind::Function(f) => Some(&f.name),
            grammar::ItemKind::Global(g) => Some(&g.name),
            grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => None,
        });

    quote! {
        pub struct Definition {
            #(#definition_fields),*
        }

        impl Definition {
            pub fn import_into(
                &self,
                module: &mut ::eisheth::module::builder::ModuleBuilder,
            ) -> ImportedDefinition {
                #(#item_imports);*

                ImportedDefinition {
                    #(#imported_item_names),*
                }
            }
        }
    }
}

fn make_define_function<'a>(
    module_name: &Ident,
    items: impl Iterator<Item = &'a Item> + Clone,
) -> proc_macro2::TokenStream {
    let name_str = module_name.to_string();

    let item_names = items.clone().filter_map(|x| match &x.kind {
        grammar::ItemKind::Function(f) => Some(&f.name),
        grammar::ItemKind::Global(g) => Some(&g.name),
        grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => None,
    });

    let item_definitions = items.map(|x| match &x.kind {
        grammar::ItemKind::Function(f) => {
            // TODO pass &f as a whole instead of name and contents separately
            make_module_function_definition(x.visibility, &f.name, &f.kind)
        }
        grammar::ItemKind::Global(g) => make_global_declaration(x.visibility, g),
        grammar::ItemKind::GlobalInitializer(gid) => make_global_initializer(gid),
        grammar::ItemKind::GlobalFinalizer(gfd) => make_global_finalizer(gfd),
    });

    quote! {
        pub fn define(package_builder: &mut ::eisheth::package::builder::PackageBuilder) -> Definition {
            let module = package_builder.add_module(#name_str).unwrap();
            #(#item_definitions);*

            Definition {
                #(#item_names),*
            }
        }
    }
}

fn make_imported_definition_struct<'a>(
    items: impl Iterator<Item = &'a Item> + Clone,
) -> proc_macro2::TokenStream {
    let imported_definition_methods =
        items
            .clone()
            .filter(|x| x.is_exported())
            .filter_map(|x| match &x.kind {
                grammar::ItemKind::Function(f) => {
                    // TODO just pass f as an argument
                    Some(make_module_function_caller(&f.name, &f.kind))
                }
                grammar::ItemKind::Global(g) => Some(make_global_getter(g)),
                grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => {
                    None
                }
            });

    let imported_definition_fields =
        items
            .filter(|x| x.is_exported())
            .filter_map(|x| match &x.kind {
                grammar::ItemKind::Function(f) => {
                    let name = &f.name;

                    Some(quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor })
                }
                grammar::ItemKind::Global(g) => {
                    let name = &g.name;

                    Some(quote! { #name: ::eisheth::module::DeclaredGlobalDescriptor })
                }
                grammar::ItemKind::GlobalInitializer(_) | grammar::ItemKind::GlobalFinalizer(_) => {
                    None
                }
            });
    quote! {
        pub struct ImportedDefinition {
            #(#imported_definition_fields),*
        }

        impl ImportedDefinition {
            #(#imported_definition_methods)*
        }
    }
}

pub fn define_module_inner(tokens: TokenStream) -> TokenStream {
    let content = parse_macro_input!(tokens as DefineModuleInput);

    let definition_struct = make_definition_struct(content.items.iter());
    let define_function = make_define_function(&content.name, content.items.iter());
    let imported_defintion_struct = make_imported_definition_struct(content.items.iter());

    quote! {
        #definition_struct
        #define_function
        #imported_defintion_struct
    }
    .into()
}
