use std::{collections::HashMap, error::Error, fmt::Display, rc::Rc};

use llvm_sys::{
    core::LLVMDisposeModule,
    linker::LLVMLinkModules2,
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{DeclaredFunctionDescriptor, ModuleId};
use crate::{
    context::diagnostic::{DIAGNOSTIC_HANDLER, Diagnostic, DiagnosticHandler},
    function::Function,
    global_symbol::GlobalSymbols,
};

#[derive(Debug)]
pub struct LinkError {
    diagnostics: Vec<Diagnostic>,
    target_module_name: String,
    source_module_name: String,
}

impl Error for LinkError {}

impl Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Failed to link {} into {}\nDiagnostics:",
            self.source_module_name, self.target_module_name
        )?;

        for diagnostic in &self.diagnostics {
            writeln!(f, "{diagnostic}")?;
        }

        Ok(())
    }
}

pub struct Module {
    id: ModuleId,
    reference: LLVMModuleRef,
    functions: HashMap<DeclaredFunctionDescriptor, LLVMValueRef>,
    symbols: Rc<GlobalSymbols>,
    global_mappings: HashMap<String, usize>,
}

impl Module {
    pub(crate) const unsafe fn new(
        id: ModuleId,
        reference: *mut llvm_sys::LLVMModule,
        functions: HashMap<DeclaredFunctionDescriptor, LLVMValueRef>,
        symbols: Rc<GlobalSymbols>,
        global_mappings: HashMap<String, usize>,
    ) -> Self {
        Self {
            id,
            reference,
            functions,
            symbols,
            global_mappings,
        }
    }

    /// # Panics
    /// If the `FunctionDeclaration` is from another module.
    #[must_use]
    pub fn get_function(&self, id: DeclaredFunctionDescriptor) -> Function<'_> {
        assert!(id.module_id == self.id);

        let function = self.functions.get(&id).unwrap();

        // SAFETY: We got a reference to the function in the HashMap, so it must be valid
        Function::new(self, *function, id.r#type)
    }

    pub(crate) fn link(&mut self, mut module: Self) -> Result<(), LinkError> {
        let reference = module.reference;

        module.reference = std::ptr::null_mut();

        // SAFETY: if the Module object exists, the reference must be valid, and we're consuming
        // the linked-in Module, so nobody can use that reference anymore
        let is_failed = unsafe { LLVMLinkModules2(self.reference, reference) } != 0;
        let diagnostics = DIAGNOSTIC_HANDLER.with(DiagnosticHandler::take_diagnostics);

        if is_failed {
            return Err(LinkError {
                diagnostics,
                target_module_name: self.symbols.resolve(self.id.1),
                source_module_name: self.symbols.resolve(module.id.1),
            });
        }

        self.global_mappings.extend(module.global_mappings.drain());

        Ok(())
    }

    pub(crate) fn symbols(&self) -> Rc<GlobalSymbols> {
        self.symbols.clone()
    }

    pub(crate) fn take(mut self) -> (HashMap<String, usize>, LLVMModuleRef) {
        let module_reference = self.reference;
        self.reference = std::ptr::null_mut();
        (self.global_mappings.drain().collect(), module_reference)
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
