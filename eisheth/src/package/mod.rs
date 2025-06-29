pub mod builder;
pub(crate) mod context;
pub(crate) mod id;

use std::rc::Rc;

use super::{global_symbol::GlobalSymbols, module::built::Module};

pub struct Package {
    module: Module,
}

impl Package {
    pub(crate) const fn new(module: Module) -> Self {
        Self { module }
    }

    pub(crate) fn into_module(self) -> Module {
        self.module
    }

    pub(crate) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.module.symbols()
    }
}
