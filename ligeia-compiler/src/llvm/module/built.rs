use std::collections::HashMap;

use llvm_sys::{
    core::LLVMDisposeModule,
    linker::LLVMLinkModules2,
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{FunctionId, ModuleId};
use crate::llvm::function::Function;

// TODO use it!
#[allow(unused)]
pub struct Module {
    id: ModuleId,
    reference: LLVMModuleRef,
    functions: HashMap<FunctionId, LLVMValueRef>,
}

impl Module {
    pub(crate) fn into_llvm_ref(mut self) -> *mut llvm_sys::LLVMModule {
        let result = self.reference;
        self.reference = std::ptr::null_mut();

        result
    }

    pub(in crate::llvm) const unsafe fn new(
        id: ModuleId,
        reference: *mut llvm_sys::LLVMModule,
        functions: HashMap<FunctionId, LLVMValueRef>,
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
        assert!(id.module_id == self.id);

        let function = self.functions.get(&id).unwrap();

        // SAFETY: We got a reference to the function in the HashMap, so it must be valid
        unsafe { Function::new(self, *function, id.r#type) }
    }

    pub(crate) fn link(&self, mut module: Self) {
        let reference = module.reference;
        module.reference = std::ptr::null_mut();
        // TODO handle errors
        // SAFETY: if the Module object exists, the reference must be valid, and we're consuming
        // the linked-in Module, so nobody can use that reference anymore
        unsafe { LLVMLinkModules2(self.reference, reference) };
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
