pub mod function;

use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    rc::Rc,
    str::FromStr,
    sync::LazyLock,
};

use function::JitFunction;
use llvm_sys::{
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMExecutionEngineRef,
        LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMRunStaticConstructors,
        LLVMRunStaticDestructors,
    },
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
};

use super::{global_symbol::GlobalSymbols, module::DeclaredFunctionDescriptor, package::Package};

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
    _token: JITToken,
    execution_engine: LLVMExecutionEngineRef,
    symbols: Rc<GlobalSymbols>,
}

impl Jit {
    /// # Panics
    /// Will panic if the execution engine cannot be crated.
    /// TODO Return an error instead of panicing in that case.
    #[must_use]
    pub fn new(package: Package) -> Self {
        let token = *JIT_SETUP;
        let symbols = package.symbols();
        let module = package.into_module();

        let execution_engine = {
            let mut engine = MaybeUninit::uninit();
            let mut error = std::ptr::null_mut();

            // SAFETY: the `module` must be correctly initialized if it exists, engine and error
            // are initialized by the called function
            if unsafe {
                LLVMCreateExecutionEngineForModule(
                    engine.as_mut_ptr(),
                    module.into_llvm_ref(),
                    &raw mut error,
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

        // SAFETY: We just initialized the engine, it's valid and ready to run static constructors
        unsafe { LLVMRunStaticConstructors(execution_engine) };

        Self {
            _token: token,
            execution_engine,
            symbols,
        }
    }

    /// # Panics
    /// If the name of the function cannot be converted to a `CString`
    /// # Safety
    /// The caller must ensure that the signature on the Rust side matches the signature of the
    /// defined function, and that the function itself is memory-safe.
    #[must_use]
    pub unsafe fn get_function<TFunction>(
        &self,
        id: DeclaredFunctionDescriptor,
    ) -> JitFunction<TFunction> {
        let name = CString::from_str(&self.symbols.resolve(id.name())).unwrap();

        // SAFETY: We have a valid `execution_engine`, valid null-terminated name. The function
        // must exist. We transmute the pointer into a generic fn() one, which must be further
        // transmuted by the callee to match the function signature
        unsafe {
            let function_address = LLVMGetFunctionAddress(self.execution_engine, name.as_ptr());
            JitFunction::new(usize::try_from(function_address).unwrap())
        }
    }
}

impl Drop for Jit {
    fn drop(&mut self) {
        // SAFETY: We didn't dispose the execution_engine, so we can run the destructors
        unsafe { LLVMRunStaticDestructors(self.execution_engine) };
        // SAFETY: If Jit is dropped, then nobody should be executing any JITted code anymore, so
        // we are free to drop it.
        unsafe { LLVMDisposeExecutionEngine(self.execution_engine) };
    }
}
