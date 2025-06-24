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
    module::{FunctionId, ModuleBuilder},
    types::{integer::U64, value::Value},
};

#[non_exhaustive]
pub struct TerminatorToken;

pub struct InstructionBuilder<'symbols, 'function, 'module> {
    builder: LLVMBuilderRef,
    function_builder: &'function FunctionBuilder<'symbols, 'module>,
    _phantom: PhantomData<&'function FunctionBlock<'symbols, 'function, 'module>>,
}

impl<'symbols, 'function, 'module> InstructionBuilder<'symbols, 'function, 'module> {
    pub(crate) fn new(block: &'function FunctionBlock<'symbols, 'function, 'module>) -> Self {
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

    pub(crate) fn add(&self, left: &Value<U64>, right: &Value<U64>, name: &str) -> Value<U64> {
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

    pub(crate) fn direct_call(&self, function: FunctionId, name: &str) -> Value<U64> {
        let name = CString::from_str(name).unwrap();
        // TODO we should get the type together with the function, and also verify the return type
        // matches the expected one
        let (function_value, function_type) = self.module().get_function(function);
        let mut arguments = vec![];

        // SAFETY: we ensured all the references are valid
        let result = unsafe {
            LLVMBuildCall2(
                self.builder,
                function_type,
                function_value,
                arguments.as_mut_ptr(),
                u32::try_from(arguments.len()).unwrap(),
                name.as_ptr(),
            )
        };

        // SAFETY: LLVMBuildCall2 will return a correct value
        // TODO ensure that we return a value of the correct type
        unsafe { Value::new(result) }
    }

    pub(crate) fn r#return(&self, sum: &Value<U64>) -> TerminatorToken {
        // SAFETY: we've a valid, positioned builder and the value must exist at least for the
        // duration of the call, so we're good
        unsafe { LLVMBuildRet(self.builder, sum.as_llvm_ref()) };

        TerminatorToken
    }

    const fn module(&self) -> &ModuleBuilder<'symbols> {
        self.function_builder.module()
    }
}

impl Drop for InstructionBuilder<'_, '_, '_> {
    fn drop(&mut self) {
        // SAFETY: We own the builder, we're free to dispose it. If anyone needs it, they should
        // have a ref to `InstructionBuilder` and prevent the Drop
        unsafe { LLVMDisposeBuilder(self.builder) };
    }
}
