pub mod type_check;

use thiserror::Error;

use crate::{analysis::type_check::TypeCheckError, parser::ast};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnalysisError {
    #[error("type check failed: {0}")]
    TypeCheck(#[from] TypeCheckError),
}

/// # Errors
/// Returns an error if the program is not well-formed
pub fn analyse(ast: &[ast::SourceFile]) -> Result<(), AnalysisError> {
    type_check::type_check(ast)?;

    Ok(())
}
