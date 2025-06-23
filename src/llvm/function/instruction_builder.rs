use std::{ffi::CString, str::FromStr};

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildRet, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMBuilderRef,
};

use super::block::FunctionBlock;
use crate::llvm::{
    LLVM_CONTEXT,
    types::{integer::U64, value::Value},
};

/// This struct has a field, so it cannot be constructed by everyone, even though it's public, and
/// the constructor's visibility decides who can construct it.
/// TODO: Is there a less awkward way to express those visibility requirements?
pub struct TerminatorToken(());

impl TerminatorToken {
    pub(in crate::llvm) fn new() -> Self {
        TerminatorToken(())
    }
}

pub struct InstructionBuilder<'function, 'module> {
    builder: LLVMBuilderRef,
    #[allow(unused)] // TODO should this be PhantomData?
    block: &'function FunctionBlock<'function, 'module>,
}

impl<'function, 'module> InstructionBuilder<'function, 'module> {
    pub(crate) fn new(block: &'function FunctionBlock<'function, 'module>) -> Self {
        let builder = LLVM_CONTEXT
            // SAFETY: The context lives for 'static so we're free to keep the builder
            .with(|context| unsafe { LLVMCreateBuilderInContext(context.as_llvm_ref()) });
        // SAFETY: we've just constructed the builder so it's valid, the block also must be
        unsafe { LLVMPositionBuilderAtEnd(builder, block.as_llvm_ref()) };

        Self { builder, block }
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

    pub(crate) fn r#return(&self, sum: &Value<U64>) -> TerminatorToken {
        // SAFETY: we've a valid, positioned builder and the value must exist at least for the
        // duration of the call, so we're good
        unsafe { LLVMBuildRet(self.builder, sum.as_llvm_ref()) };

        TerminatorToken::new()
    }
}

impl Drop for InstructionBuilder<'_, '_> {
    fn drop(&mut self) {
        // SAFETY: We own the builder, we're free to dispose it. If anyone needs it, they should
        // have a ref to `InstructionBuilder` and prevent the Drop
        unsafe { LLVMDisposeBuilder(self.builder) };
    }
}
