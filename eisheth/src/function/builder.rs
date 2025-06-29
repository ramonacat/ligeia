use std::{ffi::CString, str::FromStr as _};

use llvm_sys::{
    LLVMLinkage,
    core::{LLVMAddFunction, LLVMGetParam, LLVMSetLinkage, LLVMTypeOf},
    prelude::LLVMValueRef,
};

use super::{
    block::FunctionBlock,
    declaration::{FunctionDeclarationDescriptor, Visibility},
};
use crate::{
    module::{AnyModule, builder::ModuleBuilder},
    types::{self, Type, function::Function, value::Value},
};

pub(crate) struct FunctionReference<'module> {
    // TODO should this be PhantomData instead? we only care about the lifetime ATM
    _module: &'module dyn AnyModule,
    reference: LLVMValueRef,
    r#type: Function,
}

impl<'module> FunctionReference<'module> {
    pub(crate) unsafe fn new(
        module: &'module dyn AnyModule,
        reference: LLVMValueRef,
        r#type: Function,
    ) -> Self {
        Self {
            _module: module,
            reference,
            r#type,
        }
    }

    pub(crate) const fn r#type(&self) -> Function {
        self.r#type
    }

    pub(crate) const fn value(&self) -> LLVMValueRef {
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
    /// TODO Is this even a sensible check? Can this ever happen?
    /// If the received argument's type does not match the declared type.
    #[must_use]
    pub fn get_argument(&self, index: u32) -> Option<Value> {
        let argument_type = self.r#type.get_argument(index as usize)?;

        // SAFETY: We've ensured that the `index` is not out-of-bounds while getting the argument
        // type, and self.function is a valid reference to the function
        let argument = unsafe { LLVMGetParam(self.function, index) };

        // SAFETY: We've ensured LLVMGetParam got correct arguments, so `argument` must be valid
        assert!(argument_type == unsafe { LLVMTypeOf(argument) });

        // SAFETY: We know that the type of the argument matches the type of the Value, so this is
        // correct and safe
        Some(unsafe { Value::new(argument) })
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
