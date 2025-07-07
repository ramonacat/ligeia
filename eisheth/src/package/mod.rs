pub mod builder;
pub(crate) mod context;
pub(crate) mod id;

use std::{collections::HashMap, rc::Rc};

use super::{global_symbol::GlobalSymbols, module::built::Module};
use crate::module::AnyModuleExtensions;

pub struct Package {
    module: Module,
    ir_per_module: HashMap<String, String>,
}

impl Package {
    pub(crate) const fn new(module: Module, ir_per_module: HashMap<String, String>) -> Self {
        Self {
            module,
            ir_per_module,
        }
    }

    pub(crate) fn into_module(self) -> Module {
        self.module
    }

    pub(crate) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.module.symbols()
    }

    #[must_use]
    pub const fn ir_per_module(&self) -> &HashMap<String, String> {
        &self.ir_per_module
    }

    #[must_use]
    pub fn final_ir(&self) -> String {
        self.module.dump_ir()
    }
}
