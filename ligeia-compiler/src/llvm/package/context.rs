use std::rc::Rc;

use super::id::PackageId;
use crate::llvm::global_symbol::GlobalSymbols;

#[derive(Clone)]
pub(in crate::llvm) struct PackageContext {
    id: PackageId,
    symbols: Rc<GlobalSymbols>,
}

impl PackageContext {
    pub(in crate::llvm) const fn new(id: PackageId, symbols: Rc<GlobalSymbols>) -> Self {
        Self { id, symbols }
    }

    pub(in crate::llvm) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.symbols.clone()
    }

    pub(in crate::llvm) const fn id(&self) -> PackageId {
        self.id
    }
}
