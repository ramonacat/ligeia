use std::{ffi::CString, marker::PhantomData, str::FromStr};

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildCall2, LLVMBuildRet, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use super::{block::FunctionBlock, builder::FunctionBuilder};
use crate::llvm::{
    LLVM_CONTEXT,
    module::{FunctionDeclaration, builder::ModuleBuilder},
    types::{Type, value::Value},
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

    pub(crate) fn add(&self, left: &Value, right: &Value, name: &str) -> Value {
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
        unsafe { Value::new(value) }
    }

    pub(crate) fn direct_call(
        &self,
        function: FunctionDeclaration,
        arguments: &[Value],
        name: &str,
    ) -> Value {
        let name = CString::from_str(name).unwrap();
        let function = self.module().get_function(function);
        let mut arguments: Vec<_> = arguments.iter().map(Value::as_llvm_ref).collect();

        // SAFETY: we ensured all the references are valid
        let result = unsafe {
            LLVMBuildCall2(
                self.builder,
                function.r#type().as_llvm_ref(),
                function.value(),
                arguments.as_mut_ptr(),
                u32::try_from(arguments.len()).unwrap(),
                name.as_ptr(),
            )
        };

        // SAFETY: LLVMBuildCall2 will return a value that is valid
        unsafe { Value::new(result) }
    }

    pub(crate) fn r#return(&self, sum: &Value) -> TerminatorToken {
        // SAFETY: we've a valid, positioned builder and the value must exist at least for the
        // duration of the call, so we're good
        unsafe { LLVMBuildRet(self.builder, sum.as_llvm_ref()) };

        TerminatorToken
    }

    const fn module(&self) -> &ModuleBuilder {
        self.function_builder.module()
    }
}

impl Drop for InstructionBuilder<'_> {
    fn drop(&mut self) {
        // SAFETY: We own the builder, we're free to dispose it. If anyone needs it, they should
        // have a ref to `InstructionBuilder` and prevent the Drop
        unsafe { LLVMDisposeBuilder(self.builder) };
    }
}
