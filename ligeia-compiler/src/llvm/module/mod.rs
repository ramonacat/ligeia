pub mod built;

use std::{collections::HashMap, ffi::CString, str::FromStr};

use built::Module;
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{LLVMAddFunction, LLVMDisposeModule, LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::{LLVMModuleRef, LLVMValueRef},
};

use super::{
    LLVM_CONTEXT,
    function::builder::{FunctionBuilder, FunctionReference},
    global_symbol::{GlobalSymbol, GlobalSymbols},
    types::{self, Type, function::FunctionType},
};

pub(in crate::llvm) trait AnyModule {}
impl AnyModule for ModuleBuilder<'_> {}
impl AnyModule for Module {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId {
    module_id: ModuleId,
    name: GlobalSymbol,
    r#type: FunctionType,
}

pub struct ModuleBuilder<'symbols> {
    id: ModuleId,
    reference: LLVMModuleRef,
    global_symbols: &'symbols GlobalSymbols,
    functions: HashMap<FunctionId, LLVMValueRef>,
}

impl<'symbols> ModuleBuilder<'symbols> {
    pub fn new(global_symbols: &'symbols GlobalSymbols, name: &str) -> Self {
        let module = LLVM_CONTEXT.with(|context| {
            let name = CString::from_str(name).unwrap();

            // SAFETY: The `name` is a valid null-terminated string, and we have a reference to
            // context, so the one returned from `as_llvm_ref` must be valid
            unsafe {
                LLVMModuleCreateWithNameInContext(name.as_ptr().cast(), context.as_llvm_ref())
            }
        });

        Self {
            reference: module,
            id: ModuleId(global_symbols.intern(name)),
            global_symbols,
            functions: HashMap::new(),
        }
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> LLVMModuleRef {
        self.reference
    }

    pub(crate) fn define_function(
        &mut self,
        name: &str,
        r#type: types::function::FunctionType,
        implement: impl FnOnce(&FunctionBuilder),
    ) -> FunctionId {
        let id = FunctionId {
            module_id: self.id,
            name: self.global_symbols.intern(name),
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

    pub(crate) fn import_function(&mut self, id: FunctionId) -> FunctionId {
        assert!(id.module_id != self.id);

        let name = self.global_symbols.resolve(id.name);
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

    pub(crate) fn build(mut self) -> Module {
        let mut out_message = std::ptr::null_mut();
        // SAFETY: We have a valid, non-null `reference`, and since the action is
        // `LLVMAbortProcessAction`, and `out_message` is passed as a pointer to a pointer, so
        // we'll get a new pointer put into there
        unsafe {
            LLVMVerifyModule(
                self.reference,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                &raw mut out_message,
            )
        };

        // SAFETY: We have a valid, non-null `reference`, so this function can't fail
        unsafe { LLVMDumpModule(self.reference) };

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        let mut functions = HashMap::new();
        std::mem::swap(&mut functions, &mut self.functions);

        // SAFETY: We have ensured that the reference is not owned by this current object
        unsafe { Module::new(self.id, reference, functions) }
    }

    pub(in crate::llvm) fn get_function(&self, function: FunctionId) -> FunctionReference {
        let value = self.functions.get(&function).unwrap();

        // SAFETY: The functions here were transfered from the ModuleBuilder, so we know they
        // belong to this module, so as long as the function reference has a life time at least
        // equivalent to the lifetime of the Module, the value will remain valid
        unsafe { FunctionReference::new(self, *value, function.r#type) }
    }
}

impl Drop for ModuleBuilder<'_> {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: if `reference` is not null, we own the module and are free to dispose it
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
