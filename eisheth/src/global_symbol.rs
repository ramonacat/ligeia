use std::sync::RwLock;

use string_interner::{DefaultStringInterner, symbol::SymbolU32};

pub(crate) struct GlobalSymbols {
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

    // TODO can we somehow do this in a way that allows returning &str?
    pub(crate) fn resolve(&self, name: GlobalSymbol) -> String {
        self.interner
            .read()
            .unwrap()
            .resolve(name.0)
            .unwrap()
            .to_string()
    }
}
