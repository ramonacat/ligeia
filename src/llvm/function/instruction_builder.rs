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

pub struct InstructionBuilder<'function, 'module> {
    builder: LLVMBuilderRef,
    #[allow(unused)] // TODO should this be PhantomData?
    block: &'function FunctionBlock<'function, 'module>,
}

impl<'function, 'module> InstructionBuilder<'function, 'module> {
    pub(crate) fn new(block: &'function FunctionBlock<'function, 'module>) -> Self {
        let builder = LLVM_CONTEXT
            .with(|context| unsafe { LLVMCreateBuilderInContext(context.as_llvm_ref()) });
        unsafe { LLVMPositionBuilderAtEnd(builder, block.as_llvm_ref()) };

        Self { builder, block }
    }

    pub(crate) fn add(&self, left: &Value<U64>, right: &Value<U64>, name: &str) -> Value<U64> {
        let name = CString::from_str(name).unwrap();

        let value = unsafe {
            LLVMBuildAdd(
                self.builder,
                left.as_llvm_ref(),
                right.as_llvm_ref(),
                name.as_ptr(),
            )
        };
        unsafe { Value::new(value) }
    }

    pub(crate) fn r#return(&self, sum: &Value<U64>) {
        unsafe { LLVMBuildRet(self.builder, sum.as_llvm_ref()) };
    }
}

impl Drop for InstructionBuilder<'_, '_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.builder) };
    }
}
