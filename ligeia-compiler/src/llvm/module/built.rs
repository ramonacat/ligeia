use std::collections::HashMap;

use llvm_sys::{core::LLVMDisposeModule, prelude::{LLVMModuleRef, LLVMValueRef}};

use super::{FunctionId, ModuleId};
use crate::llvm::{function::Function, types::function::FunctionType};

// TODO use it!
#[allow(unused)]
pub struct Module {
    id: ModuleId,
    reference: LLVMModuleRef,
    functions: HashMap<FunctionId, (LLVMValueRef, FunctionType)>,
}

impl Module {
    pub(crate) fn into_llvm_ref(mut self) -> *mut llvm_sys::LLVMModule {
        let result = self.reference;
        self.reference = std::ptr::null_mut();

        result
    }

    pub(in crate::llvm) unsafe fn new(
        id: ModuleId,
        reference: *mut llvm_sys::LLVMModule,
        functions: HashMap<FunctionId, (LLVMValueRef, FunctionType)>,
    ) -> Self {
        Self {
            id,
            reference,
            functions,
        }
    }

    // TODO use it!
    #[allow(unused)]
    pub fn get_function(&self, id: FunctionId) -> Function {
        assert!(id.0 == self.id);

        let function = self.functions.get(&id).unwrap();

        // SAFETY: We got a reference to the function in the HashMap, so it must be valid
        unsafe { Function::new(self, function.0, &function.1) }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: We own the module, we're free to dispose of it, everyone who depends on it should have a
        // reference to this `BuiltModule` or take ownership with `into_llvm_ref`
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
