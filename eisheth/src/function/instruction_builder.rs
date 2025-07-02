use std::{ffi::CString, marker::PhantomData, str::FromStr};

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildArrayMalloc, LLVMBuildCall2, LLVMBuildMalloc, LLVMBuildRet,
        LLVMBuildRetVoid, LLVMBuildStore, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use super::{block::FunctionBlock, builder::FunctionBuilder};
use crate::{
    LLVM_CONTEXT,
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    types::Type,
    value::{ConstOrDynamicValue, DynamicValue, Value},
};

#[non_exhaustive]
pub struct TerminatorToken;

pub struct InstructionBuilder<'module> {
    builder: LLVMBuilderRef,
    function_builder: &'module FunctionBuilder<'module>,
    _phantom: PhantomData<&'module FunctionBlock<'module>>,
}

impl<'module> InstructionBuilder<'module> {
    pub(crate) fn new(block: &'module FunctionBlock<'module>) -> Self {
        let builder = LLVM_CONTEXT
            // SAFETY: The context lives for 'static so we're free to keep the builder
            .with(|context| unsafe { LLVMCreateBuilderInContext(context.as_llvm_ref()) });
        // SAFETY: we've just constructed the builder so it's valid, the block also must be
        unsafe { LLVMPositionBuilderAtEnd(builder, block.as_llvm_ref()) };

        Self {
            builder,
            function_builder: block.function_builder(),
            _phantom: PhantomData,
        }
    }

    /// # Panics
    /// Can panic if the name cannot be converted to a `CString`
    #[must_use]
    pub fn add(&self, left: &dyn Value, right: &dyn Value, name: &str) -> ConstOrDynamicValue {
        let name = CString::from_str(name).unwrap();

        // SAFETY: the builder is valid and positioned, left and right exist for duration of the
        // call, and name is a valid null-terminated C-string
        let value = unsafe {
            LLVMBuildAdd(
                self.builder,
                left.as_llvm_ref(),
                right.as_llvm_ref(),
                name.as_ptr(),
            )
        };
        // SAFETY: We know the types of the arguments, so the return type must match them
        unsafe { ConstOrDynamicValue::new(value) }
    }

    /// # Panics
    /// Can panic if the name cannot be converted to a `CString`
    pub fn direct_call(
        &self,
        function: DeclaredFunctionDescriptor,
        arguments: &[&dyn Value],
        name: &str,
    ) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        let function = self.module().get_function(function);
        let mut arguments: Vec<_> = arguments.iter().map(|x| x.as_llvm_ref()).collect();

        // SAFETY: we ensured all the references are valid
        let result = unsafe {
            LLVMBuildCall2(
                self.builder,
                function.r#type().as_llvm_ref(),
                function.as_llvm_ref(),
                arguments.as_mut_ptr(),
                u32::try_from(arguments.len()).unwrap(),
                name.as_ptr(),
            )
        };

        // SAFETY: LLVMBuildCall2 will return a value that is valid
        unsafe { DynamicValue::new(result) }
    }

    /// # Panics
    /// Can panic if the name cannot be converted to a `CString`
    pub fn malloc(&self, r#type: &dyn Type, name: &str) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: All the pointers come from wrappers ensuring their validity
        let value = unsafe { LLVMBuildMalloc(self.builder, r#type.as_llvm_ref(), name.as_ptr()) };

        // SAFETY: We just crated the value, it must be valid
        unsafe { DynamicValue::new(value) }
    }

    /// # Panics
    /// Can panic if the name cannot be converted to a `CString`
    pub fn malloc_array(&self, r#type: &dyn Type, length: &dyn Value, name: &str) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: All pointers come from wrappers ensuring their validity
        let value = unsafe {
            LLVMBuildArrayMalloc(
                self.builder,
                r#type.as_llvm_ref(),
                length.as_llvm_ref(),
                name.as_ptr(),
            )
        };

        // SAFETY: We just crated the value, the pointer is valid
        unsafe { DynamicValue::new(value) }
    }

    pub fn store(&self, target_pointer: &dyn Value, value: &dyn Value) {
        // SAFETY: All the pointers come from safe wrappers that ensure they're valid
        unsafe {
            LLVMBuildStore(
                self.builder,
                value.as_llvm_ref(),
                target_pointer.as_llvm_ref(),
            )
        };
    }

    #[must_use]
    pub fn r#return(&self, value: Option<&dyn Value>) -> TerminatorToken {
        if let Some(value) = value {
            // SAFETY: we've a valid, positioned builder and the value must exist at least for the
            // duration of the call, so we're good
            unsafe { LLVMBuildRet(self.builder, value.as_llvm_ref()) };
        } else {
            // SAFETY: we have a valid positioned builder
            unsafe { LLVMBuildRetVoid(self.builder) };
        }

        TerminatorToken
    }

    const fn module(&self) -> &ModuleBuilder {
        self.function_builder.module()
    }

    pub(crate) const fn builder(&self) -> LLVMBuilderRef {
        self.builder
    }
}

impl Drop for InstructionBuilder<'_> {
    fn drop(&mut self) {
        // SAFETY: We own the builder, we're free to dispose it. If anyone needs it, they should
        // have a ref to `InstructionBuilder` and prevent the Drop
        unsafe { LLVMDisposeBuilder(self.builder) };
    }
}
