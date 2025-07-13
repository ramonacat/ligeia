use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{Ident, ReturnType};

use crate::{
    convert_case,
    items::modules::grammar::{
        self, FunctionArgument, FunctionDefinitionKind, FunctionSignature, Visibility,
    },
};

pub fn make_module_function_definition(
    visibility: Visibility,
    name: &Ident,
    definition: &FunctionDefinitionKind,
) -> proc_macro2::TokenStream {
    match &definition {
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
                    let name = &x.name;
                    quote! { #name }
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

pub fn make_module_function_caller(
    name: &Ident,
    definition: &FunctionDefinitionKind,
) -> proc_macro2::TokenStream {
    let (arguments, return_type) = match definition {
        FunctionDefinitionKind::Runtime(x) => (&x.signature.arguments, &x.signature.return_type),
        FunctionDefinitionKind::Builder(x) => (&x.signature.arguments, &x.signature.return_type),
    };

    let argument_type_variables = arguments
        .iter()
        .filter_map(|x| {
            if let FunctionArgument::Arg(a) = x {
                Some(a)
            } else {
                None
            }
        })
        .map(|x| {
            x.name
                .as_ref()
                .expect("Unnamed arguments are not allowed")
                .0
                .to_string()
        })
        .map(|x| convert_case::snake_to_pascal(&x))
        .map(|x| format_ident!("T{}", x));

    let fn_arguments = arguments
        .iter()
        .filter_map(|x| {
            if let FunctionArgument::Arg(a) = x {
                Some(a)
            } else {
                None
            }
        })
        .zip(argument_type_variables.clone())
        .map(|x| {
            (
                x.0.name
                    .as_ref()
                    .expect("Unnamed arguments are not allowed")
                    .0
                    .clone(),
                x.1,
            )
        })
        .map(|(name, generic_name)| quote! { #name : #generic_name });

    let where_items = argument_type_variables
        .clone()
        .map(|x| quote! { ::eisheth::value::ConstOrDynamicValue: From<#x> });

    let where_clause = if arguments.is_empty() {
        quote! {}
    } else {
        quote! {where #(#where_items),*}
    };

    let argument_names = arguments
        .iter()
        .filter_map(|x| {
            if let FunctionArgument::Arg(a) = x {
                Some(a)
            } else {
                None
            }
        })
        .map(|x| {
            x.name
                .as_ref()
                .expect("Unnamed arguments are not allowed")
                .0
                .clone()
        });

    let name_literal = Literal::string(&name.to_string());

    let body = match return_type {
        ReturnType::Default => {
            quote! { let _ = i.direct_call(self.#name, &[#(#argument_names .into()),*], ""); }
        }
        ReturnType::Type(_, _) => {
            quote! {
                i.direct_call(self.#name, &[#(#argument_names .into()),*], #name_literal)
            }
        }
    };

    let return_type = match return_type {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, _) => quote! { -> ::eisheth::value::DynamicValue },
    };

    quote! {
        pub fn #name<#(#argument_type_variables),*>(
            &self,
            i: &::eisheth::function::instruction_builder::InstructionBuilder,
            #(#fn_arguments),*
        ) #return_type #where_clause {
            #body
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
                <(#return_type) as ::eisheth::types::RepresentedAs>::representation(),
                &[
                    #(<(#arguments) as ::eisheth::types::RepresentedAs>::representation().into()),*
                ],
            ),
            ::eisheth::function::declaration::Visibility::#visibility,
        )
    }
}
