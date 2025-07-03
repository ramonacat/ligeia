use std::{
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use thiserror::Error;

use super::{Package, context::PackageContext, id::PACKAGE_ID_GENERATOR};
use crate::{
    context::diagnostic::DIAGNOSTIC_HANDLER,
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

impl Default for PackageBuilder {
    fn default() -> Self {
        Self::new()
    }
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

    /// # Errors
    /// Will return an error if the package already contains a module with the name given.
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

    /// # Errors
    /// This will error out and return the error for the first module that fails to build.
    /// # Panics
    /// If there are no modules in the package
    pub fn build(self) -> Result<Package, Vec<ModuleBuildError>> {
        let module_build_results = self.modules.into_values().map(ModuleBuilder::build);

        let mut module_build_errors = vec![];
        let mut built_modules = vec![];

        for module_build_result in module_build_results {
            match module_build_result {
                Ok(module) => built_modules.push(module),
                Err(error) => module_build_errors.push(error),
            }
        }

        if !module_build_errors.is_empty() {
            return Err(module_build_errors);
        }

        // TODO: Return the diagnostics to the caller so they can handle printing them
        DIAGNOSTIC_HANDLER.with(|handler| {
            for diagnostic in handler.take_diagnostics() {
                eprintln!("{diagnostic:?}");
            }
        });

        let final_module = built_modules
            .pop()
            .expect("package should contain at least a single module");

        for module in built_modules {
            final_module.link(module);
        }

        Ok(Package::new(final_module))
    }
}
