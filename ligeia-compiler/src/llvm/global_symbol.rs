use std::sync::RwLock;

use string_interner::{DefaultStringInterner, symbol::SymbolU32};

pub struct GlobalSymbols {
    interner: RwLock<DefaultStringInterner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalSymbol(SymbolU32);

impl GlobalSymbols {
    pub fn new() -> Self {
        Self {
            interner: RwLock::new(DefaultStringInterner::new()),
        }
    }

    pub(crate) fn intern(&self, name: &str) -> GlobalSymbol {
        GlobalSymbol(self.interner.write().unwrap().get_or_intern(name))
    }
}
