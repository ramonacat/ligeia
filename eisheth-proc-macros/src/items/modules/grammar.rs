use syn::{
    BareFnArg, Ident, ReturnType, Token, Type, braced, parenthesized,
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

pub struct BuilderFunctionImport {
    _caret: Caret,
    pub name: Ident,
}

impl Parse for BuilderFunctionImport {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _caret: input.parse()?,
            name: input.parse()?,
        })
    }
}

pub enum FunctionArgument {
    Import(BuilderFunctionImport),
    Arg(Box<BareFnArg>),
}

impl Parse for FunctionArgument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Caret) {
            Ok(Self::Import(input.parse()?))
        } else {
            Ok(Self::Arg(input.parse()?))
        }
    }
}

pub(super) struct FunctionSignatureDescriptor {
    pub _argument_parens: Paren,
    pub arguments: Punctuated<FunctionArgument, Token![,]>,
    pub return_type: ReturnType,
}

impl Parse for FunctionSignatureDescriptor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arguments;
        Ok(Self {
            _argument_parens: parenthesized!(arguments in input),
            arguments: arguments.parse_terminated(FunctionArgument::parse, Token![,])?,
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

pub struct BuilderFunctionDefinition {
    _builder: keywords::builder,

    pub signature: FunctionSignatureDescriptor,
}

impl Parse for BuilderFunctionDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _builder: input.parse()?,
            signature: input.parse()?,
        })
    }
}

pub struct ModuleFunctionDeclaration {
    pub name: Ident,
    pub _colon: Colon,
    pub contents: ModuleFunctionDefinition,
}

impl Parse for ModuleFunctionDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            contents: input.parse()?,
        })
    }
}

#[allow(unused)]
pub struct GlobalDeclaration {
    pub _global: keywords::global,
    pub name: Ident,
    pub _colon: Colon,
    pub r#type: Type,
}

impl Parse for GlobalDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _global: input.parse()?,
            name: input.parse()?,
            _colon: input.parse()?,
            r#type: input.parse()?,
        })
    }
}

pub enum ModuleItemKind {
    Function(ModuleFunctionDeclaration),
    Global(GlobalDeclaration),
}

impl Parse for ModuleItemKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keywords::global) {
            Ok(Self::Global(input.parse()?))
        } else {
            Ok(Self::Function(input.parse()?))
        }
    }
}

pub struct ModuleItem {
    pub visibility: Visibility,
    pub kind: ModuleItemKind,
}

impl Parse for ModuleItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            visibility: input.parse()?,
            kind: input.parse()?,
        })
    }
}

pub(super) struct DefineModuleInput {
    pub _module: keywords::module,
    pub name: Ident,
    pub _items_brackets: Brace,
    pub items: Punctuated<ModuleItem, Token![;]>,
}

impl Parse for DefineModuleInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            _module: input.parse()?,
            name: input.parse()?,
            _items_brackets: braced!(content in input),
            items: content.parse_terminated(ModuleItem::parse, Token![;])?,
        })
    }
}

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(builder);
    custom_keyword!(export);
    custom_keyword!(global);
    custom_keyword!(internal);
    custom_keyword!(module);
    custom_keyword!(runtime);
}
