use std::{
    collections::HashMap, error::Error, ffi::CString, fmt::Display, rc::Rc, str::FromStr as _,
};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{LLVMAddFunction, LLVMDisposeModule, LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{FunctionId, ModuleId, built::Module};
use crate::llvm::{
    LLVM_CONTEXT,
    function::builder::{FunctionBuilder, FunctionReference},
    global_symbol::GlobalSymbols,
    package::context::PackageContext,
    types::{self, Type as _},
};

#[derive(Debug)]
pub struct ModuleBuildError {
    module_name: String,
    message: String,
}

impl Display for ModuleBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to build the module \"{}\":\n{}",
            self.module_name, self.message
        )
    }
}

impl Error for ModuleBuildError {}

pub struct ModuleBuilder {
    id: ModuleId,
    reference: LLVMModuleRef,
    symbols: Rc<GlobalSymbols>,
    functions: HashMap<FunctionId, LLVMValueRef>,
}

impl ModuleBuilder {
    pub(in crate::llvm) fn new(package_context: &PackageContext, name: &str) -> Self {
        let module = LLVM_CONTEXT.with(|context| {
            let name = CString::from_str(name).unwrap();

            // SAFETY: The `name` is a valid null-terminated string, and we have a reference to
            // context, so the one returned from `as_llvm_ref` must be valid
            unsafe {
                LLVMModuleCreateWithNameInContext(name.as_ptr().cast(), context.as_llvm_ref())
            }
        });

        let symbols = package_context.symbols();

        Self {
            reference: module,
            id: ModuleId(package_context.id(), symbols.intern(name)),
            symbols,
            functions: HashMap::new(),
        }
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }

    // TODO support setting linkage (export, internal, etc.)
    pub(crate) fn define_function(
        &mut self,
        name: &str,
        r#type: types::Function,
        implement: impl FnOnce(&FunctionBuilder),
    ) -> FunctionId {
        let id = FunctionId {
            module_id: self.id,
            name: self.symbols.intern(name),
            r#type,
        };
        let builder = FunctionBuilder::new(self, name, r#type);

        // TODO we should probably ask the builder to
        // verify that all blocks got built with at least a terminator
        implement(&builder);

        let function = builder.build();

        self.functions.insert(id, function);

        id
    }

    // TODO verify that the other module actually exports the function
    pub(crate) fn import_function(&mut self, id: FunctionId) -> FunctionId {
        assert!(id.module_id != self.id);

        let name = self.symbols.resolve(id.name);
        let c_name = CString::from_str(&name).unwrap();

        let function =
            // SAFETY: All the passed values come from objects which uphold guarantees about the
            // pointers being valid
            unsafe { LLVMAddFunction(self.reference, c_name.as_ptr(), id.r#type.as_llvm_ref()) };

        let id = FunctionId {
            module_id: self.id,
            name: id.name,
            r#type: id.r#type,
        };

        self.functions.insert(id, function);

        id
    }

    pub(crate) fn build(mut self) -> Result<Module, ModuleBuildError> {
        let mut out_message = std::ptr::null_mut();
        // SAFETY: We have a valid, non-null `reference`, and since the action is
        // `LLVMAbortProcessAction`, and `out_message` is passed as a pointer to a pointer, so
        // we'll get a new pointer put into there
        let verify_result = unsafe {
            LLVMVerifyModule(
                self.reference,
                LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &raw mut out_message,
            )
        };

        if verify_result != 0 {
            // SAFETY: We received the message from the verify call above, it must be a valid
            // pointer
            let message = unsafe { CString::from_raw(out_message) }
                .to_str()
                .unwrap()
                .to_string();

            return Err(ModuleBuildError {
                module_name: self.symbols.resolve(self.id.1),
                message,
            });
        }

        // SAFETY: We have a valid, non-null `reference`, so this function can't fail
        unsafe { LLVMDumpModule(self.reference) };

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        let mut functions = HashMap::new();
        std::mem::swap(&mut functions, &mut self.functions);

        // SAFETY: We have ensured that the reference is not owned by this current object
        Ok(unsafe { Module::new(self.id, reference, functions, self.symbols.clone()) })
    }

    pub(in crate::llvm) fn get_function(&self, function: FunctionId) -> FunctionReference {
        let value = self.functions.get(&function).unwrap();

        // SAFETY: The functions here were transfered from the ModuleBuilder, so we know they
        // belong to this module, so as long as the function reference has a life time at least
        // equivalent to the lifetime of the Module, the value will remain valid
        unsafe { FunctionReference::new(self, *value, function.r#type) }
    }
}

impl Drop for ModuleBuilder {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: if `reference` is not null, we own the module and are free to dispose it
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
