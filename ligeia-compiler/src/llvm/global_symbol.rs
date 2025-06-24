use string_interner::{DefaultStringInterner, symbol::SymbolU32};

pub struct GlobalSymbols {
    interner: DefaultStringInterner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalSymbol(SymbolU32);

impl GlobalSymbols {
    pub fn new() -> Self {
        Self {
            interner: DefaultStringInterner::new(),
        }
    }

    pub(crate) fn intern(&mut self, name: &str) -> GlobalSymbol {
        GlobalSymbol(self.interner.get_or_intern(name))
    }
}
