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
            let mut error = unsafe { std::mem::zeroed() };

            if unsafe {
                LLVMCreateExecutionEngineForModule(
                    engine.as_mut_ptr(),
                    module.into_llvm_ref(),
                    &mut error,
                )
            } != 0
            {
                assert!(!error.is_null());
                panic!("{:?}", unsafe { CStr::from_ptr(error) });
            }

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

        unsafe {
            let function_address = LLVMGetFunctionAddress(self.execution_engine, name.as_ptr());
            std::mem::transmute(usize::try_from(function_address).unwrap())
        }
    }
}

impl Drop for Jit {
    fn drop(&mut self) {
        unsafe { LLVMDisposeExecutionEngine(self.execution_engine) };
    }
}
