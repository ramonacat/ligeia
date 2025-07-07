pub mod diagnostic;

use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose, LLVMContextSetDiagnosticHandler},
    prelude::LLVMContextRef,
};

use crate::context::diagnostic::handle_diagnostic;

thread_local! {
    pub static LLVM_CONTEXT: Context = Context::new();
}

pub struct Context(LLVMContextRef);

impl Context {
    fn new() -> Self {
        // SAFETY: There are no documented global state requirements for this function, nor ways to
        // fail
        let context = unsafe { LLVMContextCreate() };

        // SAFETY: The variables we pass will exist for the duration of the program, the
        // DiagnosticContext is allowed to be null
        unsafe {
            LLVMContextSetDiagnosticHandler(context, Some(handle_diagnostic), std::ptr::null_mut());
        };
        Self(context)
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMContextRef {
        self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // SAFETY: We own the context, and everyone using it should keep a reference to context,
        // therefore if we dispose, nobody is using it anymore
        unsafe {
            LLVMContextDispose(self.0);
        }
    }
}
