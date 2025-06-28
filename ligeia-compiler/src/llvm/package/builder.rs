use std::{
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use thiserror::Error;

use super::{Package, context::PackageContext, id::PACKAGE_ID_GENERATOR};
use crate::llvm::{
    global_symbol::GlobalSymbols,
    module::builder::{ModuleBuildError, ModuleBuilder},
};

#[derive(Debug, Error)]
pub enum AddModuleError {
    #[error("Module \"{0}\" already exists in this package")]
    AlreadyExists(String),
}

pub struct PackageBuilder {
    context: PackageContext,
    modules: HashMap<String, ModuleBuilder>,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            context: PackageContext::new(
                PACKAGE_ID_GENERATOR.next(),
                Rc::new(GlobalSymbols::new()),
            ),
            modules: HashMap::new(),
        }
    }

    pub fn add_module(
        &mut self,
        name: impl Into<String>,
    ) -> Result<&mut ModuleBuilder, AddModuleError> {
        let name: String = name.into();
        let entry = self.modules.entry(name.clone());

        if let Entry::Occupied(_) = entry {
            return Err(AddModuleError::AlreadyExists(name));
        }

        Ok(entry.or_insert_with(|| ModuleBuilder::new(&self.context, &name)))
    }

    pub(crate) fn build(self) -> Result<Package, ModuleBuildError> {
        let mut built_modules = self
            .modules
            .into_values()
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
