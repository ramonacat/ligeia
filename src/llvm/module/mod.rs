use std::{ffi::CString, str::FromStr};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{LLVMDisposeModule, LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::LLVMModuleRef,
};

use super::{LLVM_CONTEXT, function::FunctionBuilder, types};

pub struct Module {
    reference: LLVMModuleRef,
}

impl Module {
    pub fn new(name: &str) -> Self {
        let module = LLVM_CONTEXT.with(|context| {
            let name = CString::from_str(name).unwrap();

            // SAFETY: The `name` is a valid null-terminated string, and we have a reference to
            // context, so the one returned from `as_llvm_ref` must be valid
            unsafe {
                LLVMModuleCreateWithNameInContext(name.as_ptr().cast(), context.as_llvm_ref())
            }
        });

        Self { reference: module }
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }

    pub(crate) fn define_function(
        &self,
        name: &str,
        r#type: types::function::FunctionType,
        implement: impl FnOnce(&FunctionBuilder),
    ) {
        let builder = FunctionBuilder::new(self, name, r#type);

        // TODO we should probably pass the builder by ref, so that we can then actually ask it to
        // verify that all blocks got built with at least a terminator
        implement(&builder);
    }

    pub(crate) fn build(mut self) -> BuiltModule {
        let mut out_message = std::ptr::null_mut();
        // SAFETY: We have a valid, non-null `reference`, and since the action is
        // `LLVMAbortProcessAction`, and `out_message` is passed as a pointer to a pointer, so
        // we'll get a new pointer put into there
        unsafe {
            LLVMVerifyModule(
                self.reference,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                &mut out_message,
            )
        };

        // SAFETY: We have a valid, non-null `reference`, so this function can't fail
        unsafe { LLVMDumpModule(self.reference) };

        let Self { reference } = self;

        self.reference = std::ptr::null_mut();

        BuiltModule { reference }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: if `reference` is not null, we own the module and are free to dispose it
        unsafe { LLVMDisposeModule(self.reference) };
    }
}

pub struct BuiltModule {
    reference: LLVMModuleRef,
}

impl BuiltModule {
    pub(crate) fn into_llvm_ref(mut self) -> *mut llvm_sys::LLVMModule {
        let result = self.reference;
        self.reference = std::ptr::null_mut();

        result
    }
}

impl Drop for BuiltModule {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: We own the module, we're free to dispose of it, everyone who depends on it should have a
        // reference to this `BuiltModule` or take ownership with `into_llvm_ref`
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
