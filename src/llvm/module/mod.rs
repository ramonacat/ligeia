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
        implement: impl FnOnce(FunctionBuilder),
    ) {
        let builder = FunctionBuilder::new(self, name, r#type);

        implement(builder);
    }

    pub(crate) fn build(mut self) -> BuiltModule {
        let mut out_message = std::ptr::null_mut();
        unsafe {
            LLVMVerifyModule(
                self.reference,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                &mut out_message,
            )
        };

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

        unsafe { LLVMDisposeModule(self.reference) };
    }
}
