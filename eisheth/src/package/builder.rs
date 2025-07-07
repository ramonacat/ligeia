use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt::Display,
    rc::Rc,
};

use thiserror::Error;

use super::{Package, context::PackageContext, id::PACKAGE_ID_GENERATOR};
use crate::{
    global_symbol::GlobalSymbols,
    module::{
        AnyModuleExtensions,
        builder::{ModuleBuildError, ModuleBuilder},
        built::LinkError,
    },
};

#[derive(Debug, Error)]
pub enum AddModuleError {
    #[error("Module \"{0}\" already exists in this package")]
    AlreadyExists(String),
}

#[derive(Debug)]
pub enum PackageBuildError {
    Link(LinkError),
    Build(Vec<ModuleBuildError>),
}

impl Error for PackageBuildError {}

impl Display for PackageBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Link(link_error) => {
                write!(f, "Link error:\n{link_error}")
            }
            Self::Build(module_build_errors) => {
                writeln!(f, "Module build errors:")?;

                for error in module_build_errors {
                    writeln!(f, "{error}")?;
                }

                Ok(())
            }
        }
    }
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
    pub fn build(self) -> Result<Package, PackageBuildError> {
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
            return Err(PackageBuildError::Build(module_build_errors));
        }

        let ir_per_module: HashMap<String, String> = built_modules
            .iter()
            .map(|x| (x.name(), x.dump_ir()))
            .collect();

        let mut final_module = built_modules
            .pop()
            .expect("package should contain at least a single module");

        for module in built_modules {
            final_module.link(module).map_err(PackageBuildError::Link)?;
        }

        Ok(Package::new(final_module, ir_per_module))
    }
}
