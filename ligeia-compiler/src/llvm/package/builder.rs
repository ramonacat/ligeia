use std::rc::Rc;

use super::Package;
use crate::llvm::{
    global_symbol::GlobalSymbols,
    module::builder::{ModuleBuildError, ModuleBuilder},
};

pub struct PackageBuilder {
    global_symbols: Rc<GlobalSymbols>,
    modules: Vec<ModuleBuilder>,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            global_symbols: Rc::new(GlobalSymbols::new()),
            modules: vec![],
        }
    }

    pub fn add_module(&mut self, name: &str) -> &mut ModuleBuilder {
        self.modules
            .push(ModuleBuilder::new(self.global_symbols.clone(), name));

        self.modules.last_mut().unwrap()
    }

    pub(crate) fn build(self) -> Result<Package, ModuleBuildError> {
        let mut built_modules = self
            .modules
            .into_iter()
            .map(ModuleBuilder::build)
            .collect::<Result<Vec<_>, ModuleBuildError>>()?;

        let final_module = built_modules
            .pop()
            .expect("package should contain at least a single module");

        for module in built_modules {
            final_module.link(module);
        }

        Ok(Package::new(final_module))
    }
}
