pub mod function;
pub mod global_symbol;
pub mod jit;
pub mod module;
pub mod package;
pub mod types;

use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

struct Context(LLVMContextRef);

impl Context {
    fn new() -> Self {
        // SAFETY: There are no documented global state requirements for this function, nor ways to
        // fail
        Self(unsafe { LLVMContextCreate() })
    }

    const fn as_llvm_ref(&self) -> LLVMContextRef {
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

thread_local! {
    pub static LLVM_CONTEXT: Context = Context::new();
}
