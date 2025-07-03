use std::{ffi::CString, str::FromStr as _};

use llvm_sys::{
    LLVMLinkage,
    core::{LLVMAddFunction, LLVMGetParam, LLVMSetLinkage},
    prelude::LLVMValueRef,
};

use super::{
    block::FunctionBlock,
    declaration::{FunctionDeclarationDescriptor, Visibility},
};
use crate::{
    module::{AnyModule, builder::ModuleBuilder},
    types::{self, Type},
    value::DynamicValue,
};

pub(crate) struct FunctionReference<'module> {
    // TODO should this be PhantomData instead? we only care about the lifetime ATM
    _module: &'module dyn AnyModule,
    reference: LLVMValueRef,
    r#type: types::Function,
}

impl<'module> FunctionReference<'module> {
    pub(crate) unsafe fn new(
        module: &'module dyn AnyModule,
        reference: LLVMValueRef,
        r#type: types::Function,
    ) -> Self {
        Self {
            _module: module,
            reference,
            r#type,
        }
    }

    pub(crate) const fn r#type(&self) -> types::Function {
        self.r#type
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMValueRef {
        self.reference
    }
}

pub struct FunctionBuilder<'module> {
    function: LLVMValueRef,
    r#type: types::Function,
    module: &'module ModuleBuilder,
}

impl<'module> FunctionBuilder<'module> {
    /// # Panics
    /// Will panic if the function name cannot be expressed as a `CString`
    #[must_use]
    pub fn new(
        module: &'module ModuleBuilder,
        declaration: &FunctionDeclarationDescriptor,
    ) -> Self {
        let name = CString::from_str(declaration.name()).unwrap();

        let function =
        // SAFETY: The module is a valid module, the name is a null terminated string, and the type
        // exists for the duration of the call, so we're safe
            unsafe { LLVMAddFunction(
            module.as_llvm_ref(),
            name.as_ptr(),
            declaration.r#type().as_llvm_ref()
        ) };
        let linkage = match declaration.visibility() {
            Visibility::Internal => LLVMLinkage::LLVMInternalLinkage,
            Visibility::Export => LLVMLinkage::LLVMExternalLinkage,
        };
        // SAFETY: We just created the function, and the linkage is one of the correct enum values
        unsafe { LLVMSetLinkage(function, linkage) };

        Self {
            function,
            r#type: declaration.r#type(),
            module,
        }
    }

    #[must_use]
    pub fn create_block(&'module self, name: &str) -> FunctionBlock<'module> {
        FunctionBlock::new(self, name)
    }

    /// # Panics
    /// Will panic if the argument index does not fit in a u32.
    #[must_use]
    pub fn get_argument(&self, index: usize) -> Option<DynamicValue> {
        if index >= self.r#type.arguments_count() {
            return None;
        }

        // SAFETY: We've ensured that the `index` is not out-of-bounds while getting the argument
        // type, and self.function is a valid reference to the function
        let argument = unsafe { LLVMGetParam(self.function, u32::try_from(index).unwrap()) };

        // SAFETY: We know that the type of the argument matches the type of the Value, so this is
        // correct and safe
        Some(unsafe { DynamicValue::new(argument) })
    }

    pub(crate) const fn build(self) -> LLVMValueRef {
        self.function
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMValueRef {
        self.function
    }

    pub(crate) const fn module(&self) -> &'module ModuleBuilder {
        self.module
    }
}
