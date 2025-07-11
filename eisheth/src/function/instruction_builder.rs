use std::{ffi::CString, marker::PhantomData, str::FromStr};

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildArrayMalloc, LLVMBuildCall2, LLVMBuildLoad2, LLVMBuildMalloc,
        LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use super::{block::FunctionBlock, builder::FunctionBuilder};
use crate::{
    context::LLVM_CONTEXT,
    module::{DeclaredFunctionDescriptor, builder::ModuleBuilder},
    types::{OpaqueType, Type},
    value::{ConstOrDynamicValue, DynamicValue, Value, ValueReference},
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
    pub fn add<TLeft: ValueReference, TRight: ValueReference>(
        &self,
        left: TLeft,
        right: TRight,
        name: &str,
    ) -> ConstOrDynamicValue {
        let name = CString::from_str(name).unwrap();

        // SAFETY: the builder is valid and positioned, left and right exist for duration of the
        // call, and name is a valid null-terminated C-string
        let value = unsafe {
            LLVMBuildAdd(
                self.builder,
                left.value(self.module()).as_llvm_ref(),
                right.value(self.module()).as_llvm_ref(),
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
        arguments: &[ConstOrDynamicValue],
        name: &str,
    ) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        let function = self.module().get_function(function);
        let mut arguments: Vec<_> = arguments.iter().map(Value::as_llvm_ref).collect();

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
    pub fn malloc<T: Type>(&self, r#type: T, name: &str) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: All the pointers come from wrappers ensuring their validity
        let value = unsafe { LLVMBuildMalloc(self.builder, r#type.as_llvm_ref(), name.as_ptr()) };

        // SAFETY: We just crated the value, it must be valid
        unsafe { DynamicValue::new(value) }
    }

    /// # Panics
    /// Can panic if the name cannot be converted to a `CString`
    pub fn malloc_array<TLength: ValueReference, TValue: Type>(
        &self,
        r#type: TValue,
        length: TLength,
        name: &str,
    ) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: All pointers come from wrappers ensuring their validity
        let value = unsafe {
            LLVMBuildArrayMalloc(
                self.builder,
                r#type.as_llvm_ref(),
                length.value(self.module()).as_llvm_ref(),
                name.as_ptr(),
            )
        };

        // SAFETY: We just crated the value, the pointer is valid
        unsafe { DynamicValue::new(value) }
    }

    pub fn store<TTarget: ValueReference, TValue: ValueReference>(
        &self,
        target_pointer: TTarget,
        value: TValue,
    ) {
        // SAFETY: All the pointers come from safe wrappers that ensure they're valid
        unsafe {
            LLVMBuildStore(
                self.builder,
                value.value(self.module()).as_llvm_ref(),
                target_pointer.value(self.module()).as_llvm_ref(),
            )
        };
    }

    /// # Panics
    /// Will panic if the name cannpt be converted to a `CString`
    pub fn load<TPointer: ValueReference, TValue: Into<OpaqueType>>(
        &self,
        pointer: TPointer,
        r#type: TValue,
        name: &str,
    ) -> DynamicValue {
        let name = CString::from_str(name).unwrap();
        // SAFETY: all the values come from safe wrappers, so the pointers must be valid
        let result = unsafe {
            LLVMBuildLoad2(
                self.builder,
                r#type.into().as_llvm_ref(),
                pointer.value(self.module()).as_llvm_ref(),
                name.as_ptr(),
            )
        };

        // SAFETY: We just crated the value, it must be valid
        unsafe { DynamicValue::new(result) }
    }

    #[must_use]
    pub fn return_void(&self) -> TerminatorToken {
        // SAFETY: we have a valid positioned builder
        unsafe { LLVMBuildRetVoid(self.builder) };

        TerminatorToken
    }

    #[must_use]
    pub fn r#return<TValue: Value>(&self, value: TValue) -> TerminatorToken {
        // SAFETY: we've a valid, positioned builder and the value must exist at least for the
        // duration of the call, so we're good
        unsafe { LLVMBuildRet(self.builder, value.as_llvm_ref()) };

        TerminatorToken
    }

    #[must_use]
    pub const fn module(&self) -> &'module ModuleBuilder {
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
