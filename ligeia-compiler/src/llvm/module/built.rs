use std::collections::HashMap;

use llvm_sys::{
    core::LLVMDisposeModule,
    prelude::{LLVMModuleRef, LLVMTypeRef, LLVMValueRef},
};

use super::{FunctionId, ModuleId};
use crate::llvm::function::Function;

// TODO use it!
#[allow(unused)]
pub struct Module {
    id: ModuleId,
    reference: LLVMModuleRef,
    functions: HashMap<FunctionId, Function>,
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
        functions: &HashMap<FunctionId, (LLVMValueRef, LLVMTypeRef)>,
    ) -> Self {
        let functions = functions
            .iter()
            // SAFETY: The caller must provide correct pairs of the value and type
            .map(|(k, v)| (*k, unsafe { Function::new(v.0, v.1) }))
            .collect();

        Self {
            id,
            reference,
            functions,
        }
    }

    // TODO use it!
    #[allow(unused)]
    pub fn get_function(&self, id: FunctionId) -> &Function {
        assert!(id.0 == self.id);

        self.functions.get(&id).unwrap()
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
