use std::rc::Rc;

use super::id::PackageId;
use crate::global_symbol::GlobalSymbols;

#[derive(Clone)]
pub struct PackageContext {
    id: PackageId,
    symbols: Rc<GlobalSymbols>,
}

impl PackageContext {
    pub const fn new(id: PackageId, symbols: Rc<GlobalSymbols>) -> Self {
        Self { id, symbols }
    }

    pub fn symbols(&self) -> Rc<GlobalSymbols> {
        self.symbols.clone()
    }

    pub const fn id(&self) -> PackageId {
        self.id
    }
}
