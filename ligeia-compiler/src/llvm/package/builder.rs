use std::rc::Rc;

use super::{Package, context::PackageContext, id::PACKAGE_ID_GENERATOR};
use crate::llvm::{
    global_symbol::GlobalSymbols,
    module::builder::{ModuleBuildError, ModuleBuilder},
};

pub struct PackageBuilder {
    context: PackageContext,
    modules: Vec<ModuleBuilder>,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            context: PackageContext::new(
                PACKAGE_ID_GENERATOR.next(),
                Rc::new(GlobalSymbols::new()),
            ),
            modules: vec![],
        }
    }

    pub fn add_module(&mut self, name: &str) -> &mut ModuleBuilder {
        // TODO we have to assert here that there isn't already a module with the same name,
        // otherwise the ModuleIds will be non-unique
        self.modules.push(ModuleBuilder::new(&self.context, name));

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
