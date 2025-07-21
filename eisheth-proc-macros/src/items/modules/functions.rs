use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{Ident, ReturnType};

use crate::items::modules::grammar::{
    self, FunctionArgument, FunctionDefinition, FunctionDefinitionKind, FunctionSignature,
    Visibility,
};

pub fn make_function_definition(
    visibility: Visibility,
    function: &FunctionDefinition,
) -> proc_macro2::TokenStream {
    let name = &function.name;
    match &function.kind {
        FunctionDefinitionKind::Runtime(f) => {
            let argument_types = f
                .signature
                .arguments
                .iter()
                .map(|x| {
                    if let FunctionArgument::Arg(arg) = x {
                        arg
                    } else {
                        panic!("Imports are not supported in runtime functions.")
                    }
                })
                .map(|x| x.ty.clone());
            let return_type = &f.signature.return_type;

            let signature_for_cast =
                quote! { unsafe extern "C" fn(#(#argument_types),*) #return_type };

            let signature = make_function_signature(visibility, name, &f.signature);
            quote! {
                let #name = unsafe {
                    module.define_runtime_function(
                        &#signature,
                        runtime::#name as (#signature_for_cast) as usize
                    )
                };
            }
        }
        FunctionDefinitionKind::Builder(f) => {
            let argument_getters = f
                .signature
                .arguments
                .iter()
                .filter_map(|x| {
                    if let FunctionArgument::Arg(a) = x {
                        Some(a)
                    } else {
                        None
                    }
                })
                .enumerate()
                .map(|(i, _)| {
                    quote! { function.get_argument(#i).unwrap() }
                });

            let import_arguments = f
                .signature
                .arguments
                .iter()
                .filter_map(|x| {
                    if let FunctionArgument::Import(a) = x {
                        Some(a)
                    } else {
                        None
                    }
                })
                .map(|x| {
                    if let Some((ident, dot)) = &x.parent {
                        let name = format_ident!("get_{}", &x.name);

                        quote! { #ident #dot #name() }
                    } else {
                        let name = &x.name;

                        quote! { #name }
                    }
                });

            let signature = make_function_signature(visibility, name, &f.signature);

            quote! {
                let #name = module.define_function(
                    &#signature,
                    |function| {
                        builder::#name(
                            function,
                            #(#import_arguments,)*
                            #(#argument_getters,)*
                        );
                    }
                );
            }
        }
    }
}

pub fn make_function_getter(function: &FunctionDefinition) -> proc_macro2::TokenStream {
    let name = &function.name;
    let getter_name = format_ident!("get_{}", name);

    quote! {
        pub fn #getter_name<'module>(
            &self,
        ) -> ::eisheth::module::DeclaredFunctionDescriptor {
            self.#name
        }
    }
}

fn make_function_signature(
    visibility: Visibility,
    name: &Ident,
    signature: &FunctionSignature,
) -> proc_macro2::TokenStream {
    let FunctionSignature {
        _argument_parens: _,
        arguments,
        return_type,
    } = signature;

    let name_str = Literal::string(&name.to_string());
    let arguments = arguments
        .iter()
        .filter_map(|x| {
            if let FunctionArgument::Arg(a) = x {
                Some(a)
            } else {
                None
            }
        })
        .map(|x| &x.ty);
    let return_type = match &return_type {
        ReturnType::Default => None,
        ReturnType::Type(_, r#type) => Some(r#type),
    };

    let visibility = match visibility {
        grammar::Visibility::Export => quote! { Export },
        grammar::Visibility::Internal => quote! { Internal },
    };

    quote! {
        ::eisheth::function::declaration::FunctionSignature::new(
            #name_str,
            ::eisheth::types::Function::new(
                <(#return_type) as ::eisheth::types::RepresentedAs>::representation().into(),
                &[
                    #(<(#arguments) as ::eisheth::types::RepresentedAs>::representation().into()),*
                ],
            ),
            ::eisheth::Visibility::#visibility,
        )
    }
}
