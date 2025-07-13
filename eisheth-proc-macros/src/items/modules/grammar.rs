use syn::{
    BareFnArg, Ident, Lit, LitInt, Path, ReturnType, Token, Type, braced, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Brace, Caret, Colon, Comma, Dot, Paren},
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

pub struct ItemImport {
    _caret: Caret,
    pub parent: Option<(Ident, Dot)>,
    pub name: Ident,
}

impl Parse for ItemImport {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _caret: input.parse()?,
            parent: {
                if input.peek2(Dot) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
            name: input.parse()?,
        })
    }
}

pub enum FunctionArgument {
    Import(ItemImport),
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

pub(super) struct FunctionSignature {
    pub _argument_parens: Paren,
    pub arguments: Punctuated<FunctionArgument, Token![,]>,
    pub return_type: ReturnType,
}

impl Parse for FunctionSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arguments;
        Ok(Self {
            _argument_parens: parenthesized!(arguments in input),
            arguments: arguments.parse_terminated(FunctionArgument::parse, Token![,])?,
            return_type: input.parse()?,
        })
    }
}

pub enum FunctionDefinitionKind {
    Runtime(RuntimeFunctionSignature),
    Builder(BuilderFunctionSignature),
}

impl Parse for FunctionDefinitionKind {
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

pub struct RuntimeFunctionSignature {
    _runtime: keywords::runtime,
    pub signature: FunctionSignature,
}

impl Parse for RuntimeFunctionSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _runtime: input.parse()?,
            signature: input.parse()?,
        })
    }
}

pub struct BuilderFunctionSignature {
    _builder: keywords::builder,

    pub signature: FunctionSignature,
}

impl Parse for BuilderFunctionSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _builder: input.parse()?,
            signature: input.parse()?,
        })
    }
}

pub struct FunctionDefinition {
    pub name: Ident,
    pub _colon: Colon,
    pub kind: FunctionDefinitionKind,
}

impl Parse for FunctionDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            kind: input.parse()?,
        })
    }
}

pub struct GlobalDeclaration {
    pub _global: keywords::global,
    pub name: Ident,
    pub _colon: Colon,
    pub r#type: Type,
    pub value: Option<(Token![=], Lit)>,
}

impl Parse for GlobalDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _global: input.parse()?,
            name: input.parse()?,
            _colon: input.parse()?,
            r#type: input.parse()?,
            value: {
                let lookahead = input.lookahead1();

                if lookahead.peek(Token![=]) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
        })
    }
}

pub struct GlobalInitializerDeclaration {
    pub _global_initializer: keywords::global_initializer,
    pub _colon: Colon,
    pub priority: LitInt,
    pub _comma1: Comma,
    pub initializer_fn: Ident,
    pub data_pointer: Option<(Comma, Ident)>,
}

impl Parse for GlobalInitializerDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _global_initializer: input.parse()?,
            _colon: input.parse()?,
            priority: input.parse()?,
            _comma1: input.parse()?,
            initializer_fn: input.parse()?,
            data_pointer: {
                let lookahead = input.lookahead1();

                if lookahead.peek(Comma) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
        })
    }
}

pub struct GlobalFinalizerDeclaration {
    pub _global_finalizer: keywords::global_finalizer,
    pub _colon: Colon,
    pub priority: LitInt,
    pub _comma1: Comma,
    pub finalizer_fn: Ident,
    pub data_pointer: Option<(Comma, Ident)>,
}

impl Parse for GlobalFinalizerDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _global_finalizer: input.parse()?,
            _colon: input.parse()?,
            priority: input.parse()?,
            _comma1: input.parse()?,
            finalizer_fn: input.parse()?,
            data_pointer: {
                let lookahead = input.lookahead1();

                if lookahead.peek(Comma) {
                    Some((input.parse()?, input.parse()?))
                } else {
                    None
                }
            },
        })
    }
}

pub enum ItemKind {
    Function(FunctionDefinition),
    Global(GlobalDeclaration),
    GlobalInitializer(GlobalInitializerDeclaration),
    GlobalFinalizer(GlobalFinalizerDeclaration),
}

impl Parse for ItemKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keywords::global) {
            Ok(Self::Global(input.parse()?))
        } else if lookahead.peek(keywords::global_initializer) {
            Ok(Self::GlobalInitializer(input.parse()?))
        } else if lookahead.peek(keywords::global_finalizer) {
            Ok(Self::GlobalFinalizer(input.parse()?))
        } else {
            Ok(Self::Function(input.parse()?))
        }
    }
}

pub struct Item {
    pub visibility: Visibility,
    pub kind: ItemKind,
}

impl Item {
    pub fn is_exported(&self) -> bool {
        self.visibility == Visibility::Export
    }
}

impl Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            visibility: input.parse()?,
            kind: input.parse()?,
        })
    }
}

pub(super) struct ImportedModules {
    pub _import: keywords::import,
    pub _item_parenthesis: Paren,
    pub imports: Punctuated<Path, Token![,]>,
}

impl Parse for ImportedModules {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            _import: input.parse()?,
            _item_parenthesis: parenthesized!(content in input),
            imports: content.parse_terminated(Path::parse, Comma)?,
        })
    }
}

pub(super) struct DefineModuleInput {
    pub _module: keywords::module,
    pub name: Ident,
    pub imported_modules: Option<ImportedModules>,
    pub _items_brackets: Brace,
    pub items: Punctuated<Item, Token![;]>,
}

impl Parse for DefineModuleInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            _module: input.parse()?,
            name: input.parse()?,
            imported_modules: {
                let lookahead = input.lookahead1();

                if lookahead.peek(keywords::import) {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
            _items_brackets: braced!(content in input),
            items: content.parse_terminated(Item::parse, Token![;])?,
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
    custom_keyword!(global_initializer);
    custom_keyword!(global_finalizer);
    custom_keyword!(import);
}
