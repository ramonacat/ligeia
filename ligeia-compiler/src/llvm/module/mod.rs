pub mod built;

use std::{collections::HashMap, ffi::CString, str::FromStr};

use built::BuiltModule;
use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{LLVMDisposeModule, LLVMDumpModule, LLVMModuleCreateWithNameInContext},
    prelude::{LLVMModuleRef, LLVMTypeRef, LLVMValueRef},
};

use super::{
    LLVM_CONTEXT,
    function::builder::FunctionBuilder,
    global_symbol::{GlobalSymbol, GlobalSymbols},
    types::{self, Type},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(GlobalSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(ModuleId, GlobalSymbol);

pub struct Module<'symbols> {
    id: ModuleId,
    reference: LLVMModuleRef,
    global_symbols: &'symbols mut GlobalSymbols,
    functions: HashMap<FunctionId, (LLVMValueRef, LLVMTypeRef)>,
}

impl<'symbols> Module<'symbols> {
    pub fn new(global_symbols: &'symbols mut GlobalSymbols, name: &str) -> Self {
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
        let type_ref = r#type.as_llvm_ref();
        let builder = FunctionBuilder::new(self, name, r#type);

        // TODO we should probably ask the builder to
        // verify that all blocks got built with at least a terminator
        implement(&builder);

        let function = builder.build();

        let id = FunctionId(self.id, self.global_symbols.intern(name));

        self.functions.insert(id, (function, type_ref));

        id
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

        let reference = self.reference;
        self.reference = std::ptr::null_mut();

        // SAFETY: We have ensured that the reference is not owned by this current object
        unsafe { BuiltModule::new(self.id, reference, &self.functions) }
    }

    pub(crate) fn get_function(&self, function: FunctionId) -> (LLVMValueRef, LLVMTypeRef) {
        *self.functions.get(&function).unwrap()
    }
}

impl Drop for Module<'_> {
    fn drop(&mut self) {
        if self.reference.is_null() {
            return;
        }

        // SAFETY: if `reference` is not null, we own the module and are free to dispose it
        unsafe { LLVMDisposeModule(self.reference) };
    }
}
