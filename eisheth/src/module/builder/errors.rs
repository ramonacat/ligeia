use std::{error::Error, fmt::Display};

use thiserror::Error;

use crate::module::DeclaredFunctionDescriptor;

#[derive(Debug)]
pub struct ModuleBuildError {
    pub(super) module_name: String,
    pub(super) message: String,
    pub(super) diagnostics: Vec<crate::context::diagnostic::Diagnostic>,
    pub(super) raw_ir: String,
}

impl Display for ModuleBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to build the module \"{}\":\n{}\nDiagnosics:\n",
            self.module_name, self.message,
        )?;

        for diagnostic in &self.diagnostics {
            writeln!(f, "{diagnostic}")?;
        }

        writeln!(f, "LLVM IR:\n{}", self.raw_ir)?;

        Ok(())
    }
}

impl Error for ModuleBuildError {}

#[derive(Debug, Error)]
pub enum FunctionImportError {
    #[error("Function {0:?} is not exported")]
    NotExported(DeclaredFunctionDescriptor),
    #[error("Function {0:?} cannot be imported into the same module where it was defined")]
    DefinedInThisModule(DeclaredFunctionDescriptor),
}
