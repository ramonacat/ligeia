use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{
    BareFnArg, Expr, Ident, ReturnType, Token, parenthesized,
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{Caret, Comma, Paren},
};

use crate::convert_case;

#[allow(dead_code)]
struct FunctionSignatureDescriptor {
    argument_parens: Paren,
    // FnArg also can match `self`, while `PatType` is only for "normal" parameters
    arguments: Punctuated<BareFnArg, Token![,]>,
    return_type: ReturnType,
}

impl Parse for FunctionSignatureDescriptor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arguments;
        Ok(Self {
            argument_parens: parenthesized!(arguments in input),
            arguments: arguments.parse_terminated(BareFnArg::parse, Token![,])?,
            return_type: input.parse()?,
        })
    }
}

#[allow(dead_code)]
struct MakeFunctionSignature {
    name: Ident,
    comma: Comma,
    signature: FunctionSignatureDescriptor,
}

impl Parse for MakeFunctionSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            comma: input.parse()?,
            signature: input.parse()?,
        })
    }
}

fn make_function_signature(
    name: &Ident,
    signature: &FunctionSignatureDescriptor,
) -> proc_macro2::TokenStream {
    let FunctionSignatureDescriptor {
        argument_parens: _,
        arguments,
        return_type,
    } = signature;

    let name_str = Literal::string(&name.to_string());
    let arguments = arguments.iter().map(|x| &x.ty);
    let return_type = match &return_type {
        ReturnType::Default => None,
        ReturnType::Type(_, r#type) => Some(r#type),
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
            // TODO allow creating internal functions as well
            ::eisheth::function::declaration::Visibility::Export,
        )
    }
}

enum ModuleFunctionDefinition {
    Runtime(RuntimeFunctionDefintion),
    Builder(BuilderFunctionDefinition),
}

impl Parse for ModuleFunctionDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::runtime) {
            Ok(Self::Runtime(input.parse()?))
        } else if lookahead.peek(keywords::builder) {
            Ok(Self::Builder(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[allow(dead_code)]
struct RuntimeFunctionDefintion {
    runtime: keywords::runtime,
    signature: FunctionSignatureDescriptor,
}

impl Parse for RuntimeFunctionDefintion {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            runtime: input.parse()?,
            signature: input.parse()?,
        })
    }
}

#[allow(dead_code)]
struct BuilderFunctionImport {
    caret: Caret,
    name: Ident,
}

impl Parse for BuilderFunctionImport {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            caret: input.parse()?,
            name: input.parse()?,
        })
    }
}

#[allow(dead_code)]
struct BuilderFunctionDefinition {
    builder: keywords::builder,
    imports: Option<Punctuated<BuilderFunctionImport, Token![,]>>,

    signature: FunctionSignatureDescriptor,
}

impl Parse for BuilderFunctionDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            builder: input.parse()?,
            imports: if input.peek(Token![^]) {
                Some(input.parse_terminated(BuilderFunctionImport::parse, Token![,])?)
            } else {
                None
            },
            signature: input.parse()?,
        })
    }
}

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(runtime);
    custom_keyword!(builder);
}

#[allow(dead_code)]
struct DefineModuleFunctionCallerInput {
    name: Ident,
    comma: Comma,
    definition_parens: Paren,
    definition: ModuleFunctionDefinition,
}

impl Parse for DefineModuleFunctionCallerInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let definition;
        Ok(Self {
            name: input.parse()?,
            comma: input.parse()?,
            definition_parens: parenthesized!(definition in input),
            definition: definition.parse()?,
        })
    }
}

pub fn define_module_function_caller_inner(tokens: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(tokens as DefineModuleFunctionCallerInput);
    let name = definition.name;

    let (arguments, return_type) = match definition.definition {
        ModuleFunctionDefinition::Runtime(x) => (x.signature.arguments, x.signature.return_type),
        ModuleFunctionDefinition::Builder(x) => (x.signature.arguments, x.signature.return_type),
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
    .into()
}

#[allow(dead_code)]
struct DefineModuleFunctionInput {
    module: Expr,
    comma0: Comma,
    name: Ident,
    comma1: Comma,
    definition_parens: Paren,
    definition: ModuleFunctionDefinition,
}

impl Parse for DefineModuleFunctionInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let definition_content;

        Ok(Self {
            module: input.parse()?,
            comma0: input.parse()?,
            name: input.parse()?,
            comma1: input.parse()?,
            definition_parens: parenthesized!(definition_content in input),
            definition: definition_content.parse()?,
        })
    }
}

pub fn define_module_function_inner(tokens: TokenStream) -> TokenStream {
    let content = parse_macro_input!(tokens as DefineModuleFunctionInput);
    let name = content.name;
    let module = content.module;

    match &content.definition {
        ModuleFunctionDefinition::Runtime(f) => {
            let argument_types = f.signature.arguments.iter().map(|x| x.ty.clone());
            let return_type = &f.signature.return_type;

            let signature_for_cast =
                quote! { unsafe extern "C" fn(#(#argument_types),*) #return_type };

            let signature = make_function_signature(&name, &f.signature);
            quote! {
                let #name = unsafe {
                    #module.define_runtime_function(
                        &#signature,
                        runtime::#name as (#signature_for_cast) as usize
                    )
                };
            }
        }
        ModuleFunctionDefinition::Builder(f) => {
            let argument_getters = f.signature.arguments.iter().enumerate().map(|(i, _)| {
                quote! { function.get_argument(#i) }
            });

            let signature = make_function_signature(&name, &f.signature);

            quote! {
                let #name = #module.define_function(
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
    .into()
}
