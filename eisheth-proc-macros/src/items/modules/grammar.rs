use syn::{
    BareFnArg, Ident, ReturnType, Token, braced, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Brace, Caret, Colon, Paren},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Export,
    Internal,
}

impl Parse for Visibility {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keywords::export) {
            let _: keywords::export = input.parse()?;

            Ok(Self::Export)
        } else if lookahead.peek(keywords::internal) {
            let _: keywords::internal = input.parse()?;

            Ok(Self::Internal)
        } else {
            Ok(Self::Export)
        }
    }
}

pub(super) struct FunctionSignatureDescriptor {
    pub _argument_parens: Paren,
    pub arguments: Punctuated<BareFnArg, Token![,]>,
    pub return_type: ReturnType,
}

impl Parse for FunctionSignatureDescriptor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arguments;
        Ok(Self {
            _argument_parens: parenthesized!(arguments in input),
            arguments: arguments.parse_terminated(BareFnArg::parse, Token![,])?,
            return_type: input.parse()?,
        })
    }
}

pub enum ModuleFunctionDefinition {
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

pub struct RuntimeFunctionDefintion {
    _runtime: keywords::runtime,
    pub signature: FunctionSignatureDescriptor,
}

impl Parse for RuntimeFunctionDefintion {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _runtime: input.parse()?,
            signature: input.parse()?,
        })
    }
}

// TODO implement imports
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

pub struct BuilderFunctionDefinition {
    _builder: keywords::builder,
    #[allow(dead_code)] // TODO implement imports
    imports: Option<Punctuated<BuilderFunctionImport, Token![,]>>,

    pub signature: FunctionSignatureDescriptor,
}

impl Parse for BuilderFunctionDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _builder: input.parse()?,
            imports: if input.peek(Token![^]) {
                Some(input.parse_terminated(BuilderFunctionImport::parse, Token![,])?)
            } else {
                None
            },
            signature: input.parse()?,
        })
    }
}

pub struct ModuleFunctionDeclaration {
    pub visibility: Visibility,
    pub name: Ident,
    pub _colon: Colon,
    pub _contents_parenthesis: Paren,
    pub contents: ModuleFunctionDefinition,
}

impl Parse for ModuleFunctionDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            visibility: input.parse()?,
            name: input.parse()?,
            _colon: input.parse()?,
            _contents_parenthesis: parenthesized!(content in input),
            contents: content.parse()?,
        })
    }
}

pub(super) struct DefineModuleInput {
    pub _module: keywords::module,
    pub name: Ident,
    pub _functions_brackets: Brace,
    pub functions: Punctuated<ModuleFunctionDeclaration, Token![;]>,
}

impl Parse for DefineModuleInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            _module: input.parse()?,
            name: input.parse()?,
            _functions_brackets: braced!(content in input),
            functions: content.parse_terminated(ModuleFunctionDeclaration::parse, Token![;])?,
        })
    }
}

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(runtime);
    custom_keyword!(builder);
    custom_keyword!(module);
    custom_keyword!(internal);
    custom_keyword!(export);
}
