pub mod builder;

use std::rc::Rc;

use super::{global_symbol::GlobalSymbols, module::built::Module};

pub struct Package {
    module: Module,
}

impl Package {
    pub(in crate::llvm) const fn new(module: Module) -> Self {
        Self { module }
    }

    pub(in crate::llvm) fn into_module(self) -> Module {
        self.module
    }

    pub(in crate::llvm) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.module.symbols()
    }
}
