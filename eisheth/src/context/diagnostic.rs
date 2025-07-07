use std::{
    cell::RefCell,
    ffi::{CStr, c_void},
    fmt::Display,
};

use llvm_sys::{
    LLVMDiagnosticSeverity,
    core::{LLVMDisposeMessage, LLVMGetDiagInfoDescription, LLVMGetDiagInfoSeverity},
    prelude::LLVMDiagnosticInfoRef,
};

thread_local! {
    pub static DIAGNOSTIC_HANDLER: DiagnosticHandler = const {DiagnosticHandler::new()};
}

#[derive(Debug)]
enum DiagnosticSeverity {
    Error,
    Warning,
    Remark,
    Note,
}

impl From<LLVMDiagnosticSeverity> for DiagnosticSeverity {
    fn from(value: LLVMDiagnosticSeverity) -> Self {
        match value {
            LLVMDiagnosticSeverity::LLVMDSError => Self::Error,
            LLVMDiagnosticSeverity::LLVMDSWarning => Self::Warning,
            LLVMDiagnosticSeverity::LLVMDSRemark => Self::Remark,
            LLVMDiagnosticSeverity::LLVMDSNote => Self::Note,
        }
    }
}

impl Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Error => "ERROR",
                Self::Warning => "WARNING",
                Self::Remark => "REMARK",
                Self::Note => "NOTE",
            }
        )
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    message: String,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.severity, self.message)
    }
}

pub struct DiagnosticHandler {
    diagnostics: RefCell<Vec<Diagnostic>>,
}

impl DiagnosticHandler {
    const fn new() -> Self {
        Self {
            diagnostics: RefCell::new(vec![]),
        }
    }

    pub(crate) fn take_diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.borrow_mut().drain(..).collect()
    }
}

pub(super) extern "C" fn handle_diagnostic(
    diagnostic_info: LLVMDiagnosticInfoRef,
    _context: *mut c_void,
) {
    // SAFETY: LLVM will always call this with a valid pointer
    let message = unsafe { LLVMGetDiagInfoDescription(diagnostic_info) };
    // SAFETY: LLVM will always call this with a valid pointer
    let severity = unsafe { LLVMGetDiagInfoSeverity(diagnostic_info) };

    let diagnostic = Diagnostic {
        // SAFETY: We just received the pointer from a function that returns a C-string
        message: unsafe { CStr::from_ptr(message).to_str().unwrap().to_string() },
        severity: severity.into(),
    };

    // SAFETY: We just received the message, copied the contents and keep no references
    unsafe { LLVMDisposeMessage(message) };

    DIAGNOSTIC_HANDLER.with(|handler| {
        handler.diagnostics.borrow_mut().push(diagnostic);
    });
}
