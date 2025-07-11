use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{Ident, ReturnType, parse_macro_input};

use crate::{
    convert_case,
    items::modules::grammar::{
        DefineModuleInput, FunctionArgument, FunctionSignatureDescriptor, ModuleFunctionDefinition,
        Visibility,
    },
    types::rust_type_to_eisheth_type_instance,
};

mod grammar;

fn make_module_function_definition(
    visibility: Visibility,
    name: &Ident,
    definition: &ModuleFunctionDefinition,
) -> proc_macro2::TokenStream {
    match &definition {
        ModuleFunctionDefinition::Runtime(f) => {
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
        ModuleFunctionDefinition::Builder(f) => {
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
    signature: &FunctionSignatureDescriptor,
) -> proc_macro2::TokenStream {
    let FunctionSignatureDescriptor {
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

pub fn define_module_inner(tokens: TokenStream) -> TokenStream {
    let content = parse_macro_input!(tokens as DefineModuleInput);

    let name_str = content.name.to_string();

    let definition_fields = content.items.iter().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => {
            let name = &f.name;

            quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor }
        }
        grammar::ModuleItemKind::Global(g) => {
            let name = &g.name;

            quote! { #name: ::eisheth::module::DeclaredGlobalDescriptor }
        }
    });

    let exported_functions = content
        .items
        .iter()
        .filter(|x| x.visibility == Visibility::Export);

    let function_imports = exported_functions.clone().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => {
            let name = &f.name;

            quote! { let #name = module.import_function(self.#name).unwrap(); }
        }
        grammar::ModuleItemKind::Global(g) => {
            let name = &g.name;

            quote! { let #name = module.import_global(self.#name).unwrap(); }
        }
    });

    let imported_function_names = exported_functions.clone().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => &f.name,
        grammar::ModuleItemKind::Global(g) => &g.name,
    });

    let function_definitions = content.items.iter().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => {
            make_module_function_definition(x.visibility, &f.name, &f.contents)
        }
        grammar::ModuleItemKind::Global(g) => {
            let name = &g.name;
            let name_str = name.to_string();
            let r#type = rust_type_to_eisheth_type_instance(&g.r#type);

            quote! {
                // TODO support setting a const initializer
                let #name = module.define_global(#name_str, #r#type, None);
            }
        }
    });

    let function_names = content.items.iter().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => &f.name,
        grammar::ModuleItemKind::Global(g) => &g.name,
    });

    let module_function_callers = exported_functions.clone().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => make_module_function_caller(&f.name, &f.contents),
        grammar::ModuleItemKind::Global(g) => make_global_access_methods(g),
    });

    let imported_definition_fields = exported_functions.clone().map(|x| match &x.kind {
        grammar::ModuleItemKind::Function(f) => {
            let name = &f.name;

            quote! { #name: ::eisheth::module::DeclaredFunctionDescriptor }
        }
        grammar::ModuleItemKind::Global(g) => {
            let name = &g.name;

            quote! { #name: ::eisheth::module::DeclaredGlobalDescriptor }
        }
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

fn make_global_access_methods(
    declaration: &grammar::GlobalDeclaration,
) -> proc_macro2::TokenStream {
    let name = &declaration.name;
    let name_str = name.to_string();

    let load_name = format_ident!("load_{}", name);
    let store_name = format_ident!("load_{}", name);

    quote! {
        pub fn #load_name(
            &self,
            i: &::eisheth::function::instruction_builder::InstructionBuilder
        ) -> DynamicValue {
            i.load(self.#name.into(), self.#name.r#type(), #name_str)
        }

        pub fn #store_name<TValue: ::eisheth::value::Value>(
            &self,
            i: &::eisheth::function::instruction_builder::InstructionBuilder,
            value: TValue
        ) {
            i.store(self.#name.into(), value);
        }
    }
}
