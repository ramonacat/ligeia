use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Ident, ItemStruct, parse_macro_input};

use crate::{
    convert_case::{pascal_to_lower_snake, pascal_to_upper_snake},
    types::rust_type_to_eisheth_type_instance,
};

/// # Panics
/// Will panic if the struct cannot be represented as ffi compatible
/// TODO This should also have a variant where the Rust side is opaque perhaps?
pub fn ffi_struct_inner(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);

    let mut is_repr_c = false;

    for attr in &item.attrs {
        if attr.path().is_ident("repr") && attr.parse_args::<Ident>().unwrap() == "C" {
            is_repr_c = true;
        }
    }

    assert!(
        is_repr_c,
        "To be FFI-compatible, structs must have #[repr(C)]"
    );

    let Fields::Named(fields) = &item.fields else {
        panic!("Only structs with named fields are supported");
    };

    let mut declaration_fields = vec![];

    for field in &fields.named {
        let visibility = &field.vis;
        let name = field.ident.as_ref().unwrap();
        let r#type = &field.ty;

        declaration_fields.push((visibility, name, r#type));
    }

    let representation = generate_representation(&item, &declaration_fields);
    let represented_as = generate_represented_as(&item, &declaration_fields);

    quote! {
        #item
        #represented_as
        #representation
    }
    .into()
}

fn generate_represented_as(
    item: &ItemStruct,
    declaration_fields: &Vec<(&syn::Visibility, &Ident, &syn::Type)>,
) -> proc_macro2::TokenStream {
    let name = &item.ident;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let field_types: Vec<_> = declaration_fields
        .iter()
        .map(|x| rust_type_to_eisheth_type_instance(x.2))
        .map(|x| quote! { #x.into() })
        .collect();

    let type_static_name = format_ident!("{}", pascal_to_upper_snake(&name.to_string()));
    let ffi_name = pascal_to_lower_snake(&name.to_string());

    quote! {
        impl #impl_generics ::eisheth::types::RepresentedAs for #name #ty_generics #where_clause {
            type RepresentationType = ::eisheth::types::Struct;

            fn representation() -> Self::RepresentationType {
                thread_local! {
                    static #type_static_name : ::eisheth::types::Struct =
                        ::eisheth::types::Struct::new(
                            #ffi_name,
                            &[
                                #(#field_types),*
                            ]
                        );
                }

                #type_static_name.with(|x| *x)
            }
        }
    }
}

fn generate_representation(
    item: &ItemStruct,
    declaration_fields: &Vec<(&syn::Visibility, &Ident, &syn::Type)>,
) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let visibility = &item.vis;
    let repr_name = format_ident!("{}Representation", name);
    let definition_fields = declaration_fields
        .iter()
        .map(|(visibility, name, _type)| quote! { #visibility #name : TValue });
    let constructor_parameters = declaration_fields
        .iter()
        .map(|(_visibility, name, _type)| quote! { #name : TValue });
    let field_name_list = declaration_fields
        .iter()
        .map(|(_visibility, name, _type)| quote! { #name });

    quote! {
        #visibility struct #repr_name<TValue: ::eisheth::value::Value> {
            #(#definition_fields),*
        }

        impl<TValue: ::eisheth::value::Value> #repr_name<TValue> {
            #visibility fn new(#(#constructor_parameters),*) -> Self {
                Self { #(#field_name_list),* }
            }
        }
    }
}
