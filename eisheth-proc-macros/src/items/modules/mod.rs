use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{Ident, ReturnType, parse_macro_input};

use crate::{
    convert_case,
    items::modules::grammar::{
        DefineModuleInput, FunctionSignatureDescriptor, ModuleFunctionDefinition, Visibility,
    },
};

mod grammar;

fn make_module_function_definition(
    visibility: Visibility,
    name: &Ident,
    definition: &ModuleFunctionDefinition,
) -> proc_macro2::TokenStream {
    match &definition {
        ModuleFunctionDefinition::Runtime(f) => {
            let argument_types = f.signature.arguments.iter().map(|x| x.ty.clone());
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
        ModuleFunctionDefinition::Builder(f) => {
            let argument_getters = f.signature.arguments.iter().enumerate().map(|(i, _)| {
                quote! { function.get_argument(#i).unwrap() }
            });

            let signature = make_function_signature(visibility, name, &f.signature);

            quote! {
                let #name = module.define_function(
                    &#signature,
                    |function| {
                        builder::#name(
                            function,
                            #(#argument_getters),*
                        );
                    }
                );
            }
        }
    }
}

fn make_module_function_caller(
    name: &Ident,
    definition: &ModuleFunctionDefinition,
) -> proc_macro2::TokenStream {
    let (arguments, return_type) = match definition {
        ModuleFunctionDefinition::Runtime(x) => (&x.signature.arguments, &x.signature.return_type),
        ModuleFunctionDefinition::Builder(x) => (&x.signature.arguments, &x.signature.return_type),
    };

    let argument_type_variables = arguments
        .iter()
        .map(|x| {
            x.name
                .as_ref()
                .expect("Unnamed arguments are not allowed")
                .0
                .to_string()
        })
        .map(|x| convert_case::snake_to_pascal(&x))
        .map(|x| format_ident!("T{}", x));

    #[allow(unused)]
    let fn_arguments = arguments
        .iter()
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

    let argument_names = arguments.iter().map(|x| {
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
    signature: &FunctionSignatureDescriptor,
) -> proc_macro2::TokenStream {
    let FunctionSignatureDescriptor {
        _argument_parens: _,
        arguments,
        return_type,
    } = signature;

    let name_str = Literal::string(&name.to_string());
    let arguments = arguments.iter().map(|x| &x.ty);
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
                &<(#return_type) as ::eisheth::types::RepresentedAs>::representation(),
                &[
                    #(&<(#arguments) as ::eisheth::types::RepresentedAs>::representation()),*
                ],
            ),
            ::eisheth::function::declaration::Visibility::#visibility,
        )
    }
}

pub fn define_module_inner(tokens: TokenStream) -> TokenStream {
    let content = parse_macro_input!(tokens as DefineModuleInput);

    let name_str = content.name.to_string();

    let definition_fields = content.functions.iter().map(|x| {
        let name = &x.name;

        quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor }
    });

    let exported_functions = content
        .functions
        .iter()
        .filter(|x| x.visibility == Visibility::Export);

    let function_imports = exported_functions.clone().map(|x| {
        let name = &x.name;

        quote! { let #name = module.import_function(self.#name).unwrap(); }
    });

    let imported_function_names = exported_functions.clone().map(|x| &x.name);

    let function_definitions = content
        .functions
        .iter()
        .map(|x| make_module_function_definition(x.visibility, &x.name, &x.contents));

    let function_names = content.functions.iter().map(|x| &x.name);

    let module_function_callers = exported_functions
        .clone()
        .map(|x| make_module_function_caller(&x.name, &x.contents));

    let imported_definition_fields = exported_functions.clone().map(|x| {
        let name = &x.name;

        quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor }
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
                #(#function_imports);*

                ImportedDefinition {
                    #(#imported_function_names),*
                }
            }
        }

        pub fn define(package_builder: &mut ::eisheth::package::builder::PackageBuilder) -> Definition {
            let module = package_builder.add_module(#name_str).unwrap();
            #(#function_definitions);*

            Definition {
                #(#function_names),*
            }
        }

        pub struct ImportedDefinition {
            #(#imported_definition_fields),*
        }

        impl ImportedDefinition {
            #(#module_function_callers)*
        }
    }.into()
}
