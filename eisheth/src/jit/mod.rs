pub mod function;

use std::{
    error::Error,
    ffi::{CStr, CString, c_void},
    fmt::Display,
    mem::MaybeUninit,
    rc::Rc,
    str::FromStr,
    sync::LazyLock,
};

use function::JitFunction;
use llvm_sys::{
    core::{LLVMDisposeMessage, LLVMDumpModule, LLVMGetNamedFunction},
    execution_engine::{
        LLVMAddGlobalMapping, LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine,
        LLVMExecutionEngineRef, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMRunStaticConstructors,
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

#[derive(Debug)]
pub struct JitInitializationError(pub String);

impl Display for JitInitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JIT initialization error: {}", self.0)
    }
}

impl Error for JitInitializationError {}

pub struct Jit {
    _token: JITToken,
    execution_engine: LLVMExecutionEngineRef,
    symbols: Rc<GlobalSymbols>,
}

impl Jit {
    /// # Panics
    /// Will panic if the execution engine cannot be created, and no message is provided.
    /// # Errors
    /// Will return an error if th execution engine cannot be created.
    pub fn new(package: Package) -> Result<Self, JitInitializationError> {
        let token = *JIT_SETUP;
        let symbols = package.symbols();
        let module = package.into_module();

        let (global_mappings, module_reference) = module.take();

        // TODO expose an API so the user can print the IR whatever way they want, instead of
        // printing here
        eprintln!("===FINAL LINKED MODULE===");
        // SAFETY: We just got the module_reference from a safe wrapper, so it must be valid
        unsafe { LLVMDumpModule(module_reference) };

        let execution_engine = {
            let mut engine = MaybeUninit::uninit();
            let mut error_raw = std::ptr::null_mut();

            // SAFETY: the `module` must be correctly initialized if it exists, engine and error
            // are initialized by the called function
            if unsafe {
                LLVMCreateExecutionEngineForModule(
                    engine.as_mut_ptr(),
                    module_reference,
                    &raw mut error_raw,
                )
            } != 0
            {
                assert!(!error_raw.is_null());
                // SAFETY: We've checked the `error` is not null, so it must be a valid CStr
                // pointer
                let error = JitInitializationError(
                    (unsafe { CStr::from_ptr(error_raw) })
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
                // SAFETY: We're done with the string, made our copy, safe to destroy
                unsafe { LLVMDisposeMessage(error_raw) };
                return Err(error);
            }

            // SAFETY: We have checked for errors above, so the pointer must point at an initialized
            // execution engine
            unsafe { engine.assume_init() }
        };

        for (name, address) in global_mappings {
            let name = CString::from_str(&name).unwrap();
            // TODO support globals other than functions?
            // SAFETY: The module_reference is valid, as it came from a safe wrapper, we just
            // crated the name so it's also a valid pointer
            let value = unsafe { LLVMGetNamedFunction(module_reference, name.as_ptr()) };
            assert!(!value.is_null(), "Global called {name:?} not found");

            // SAFETY: The caller must ensure that the address is correct, we just got the value,
            // so it's valid (and type-matching is up to the caller). The execution_engine is
            // valid, as it was just created.
            unsafe { LLVMAddGlobalMapping(execution_engine, value, address as *mut c_void) };
        }

        // SAFETY: We just initialized the engine, it's valid and ready to run static constructors
        unsafe { LLVMRunStaticConstructors(execution_engine) };

        Ok(Self {
            _token: token,
            execution_engine,
            symbols,
        })
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
