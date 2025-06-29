use std::{collections::HashMap, rc::Rc};

use llvm_sys::{
    core::LLVMDisposeModule,
    linker::LLVMLinkModules2,
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{FunctionDeclaration, ModuleId};
use crate::{function::Function, global_symbol::GlobalSymbols};

pub struct Module {
    id: ModuleId,
    reference: LLVMModuleRef,
    functions: HashMap<FunctionDeclaration, LLVMValueRef>,
    symbols: Rc<GlobalSymbols>,
}

impl Module {
    pub(crate) fn into_llvm_ref(mut self) -> *mut llvm_sys::LLVMModule {
        let result = self.reference;
        self.reference = std::ptr::null_mut();

        result
    }

    pub(crate) const unsafe fn new(
        id: ModuleId,
        reference: *mut llvm_sys::LLVMModule,
        functions: HashMap<FunctionDeclaration, LLVMValueRef>,
        symbols: Rc<GlobalSymbols>,
    ) -> Self {
        Self {
            id,
            reference,
            functions,
            symbols,
        }
    }

    /// # Panics
    /// If the `FunctionDeclaration` is from another module.
    #[must_use]
    pub fn get_function(&self, id: FunctionDeclaration) -> Function {
        assert!(id.module_id == self.id);

        let function = self.functions.get(&id).unwrap();

        // SAFETY: We got a reference to the function in the HashMap, so it must be valid
        Function::new(self, *function, id.r#type)
    }

    pub(crate) fn link(&self, mut module: Self) {
        let reference = module.reference;
        module.reference = std::ptr::null_mut();
        // TODO add the diagnostic handler so we can get the actual error messages from the linker
        // SAFETY: if the Module object exists, the reference must be valid, and we're consuming
        // the linked-in Module, so nobody can use that reference anymore
        let is_failed = unsafe { LLVMLinkModules2(self.reference, reference) } != 0;

        assert!(!is_failed, "Linking modules failed");
    }

    pub(crate) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.symbols.clone()
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
