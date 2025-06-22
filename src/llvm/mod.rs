pub mod function;
pub mod jit;
pub mod module;
pub mod types;

use llvm_sys::{
    core::{LLVMContextCreate, LLVMContextDispose},
    prelude::LLVMContextRef,
};

struct Context(LLVMContextRef);

impl Context {
    fn new() -> Self {
        Self(unsafe { LLVMContextCreate() })
    }

    const fn as_llvm_ref(&self) -> LLVMContextRef {
        self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.0);
        }
    }
}

thread_local! {
    pub static LLVM_CONTEXT: Context = Context::new();
}
