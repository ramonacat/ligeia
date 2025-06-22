use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    str::FromStr,
    sync::LazyLock,
};

use llvm_sys::{
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMExecutionEngineRef,
        LLVMGetFunctionAddress, LLVMLinkInMCJIT,
    },
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
};

use super::module::BuiltModule;

#[derive(Clone, Copy)]
struct JITToken;

static JIT_SETUP: LazyLock<JITToken> = LazyLock::new(|| {
    // SAFETY: These functions don't really have any prerequsites, so they're fine to go
    unsafe {
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
    };

    JITToken
});

pub struct Jit {
    #[allow(unused)]
    token: JITToken,
    execution_engine: LLVMExecutionEngineRef,
}

impl Jit {
    pub(crate) fn new(module: BuiltModule) -> Self {
        let token = *JIT_SETUP;

        let execution_engine = {
            let mut engine = MaybeUninit::uninit();
            // TODO: Can we just std::ptr::null_mut() instead?
            // SAFETY: error is a pointer, so we effectively get a nullptr here
            let mut error = unsafe { std::mem::zeroed() };

            // SAFETY: the `module` must be correctly initialized if it exists, engine and error
            // are initialized by the called function
            if unsafe {
                LLVMCreateExecutionEngineForModule(
                    engine.as_mut_ptr(),
                    module.into_llvm_ref(),
                    &mut error,
                )
            } != 0
            {
                assert!(!error.is_null());
                // SAFETY: We've checked the `error` is not null, so it must be a valid CStr
                // pointer
                panic!("{:?}", unsafe { CStr::from_ptr(error) });
            }

            // SAFETY: We have checked for errors above, so the pointer must point at an initialized
            // execution engine
            unsafe { engine.assume_init() }
        };

        Self {
            token,
            execution_engine,
        }
    }

    // TODO would be cool to genericise over the function type, instead of forcing the callee to
    // cast
    pub(crate) unsafe fn get_function(&self, name: &str) -> unsafe extern "C" fn() {
        let name = CString::from_str(name).unwrap();

        // SAFETY: We have a valid `execution_engine`, valid null-terminated name. The function
        // must exist. We transmute the pointer into a generic fn() one, which must be further
        // transmuted by the callee to match the function signature
        unsafe {
            let function_address = LLVMGetFunctionAddress(self.execution_engine, name.as_ptr());
            std::mem::transmute(usize::try_from(function_address).unwrap())
        }
    }
}

impl Drop for Jit {
    fn drop(&mut self) {
        // SAFETY: If Jit is dropped, then nobody should be executing any JITted code anymore, so
        // we are free to drop it.
        unsafe { LLVMDisposeExecutionEngine(self.execution_engine) };
    }
}
