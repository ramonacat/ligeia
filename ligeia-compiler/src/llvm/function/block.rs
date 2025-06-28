use std::{ffi::CString, str::FromStr};

use llvm_sys::core::LLVMAppendBasicBlock;

use super::{
    builder::FunctionBuilder,
    instruction_builder::{InstructionBuilder, TerminatorToken},
};

pub struct FunctionBlock<'module> {
    function_builder: &'module FunctionBuilder<'module>,
    block: *mut llvm_sys::LLVMBasicBlock,
}

impl<'module> FunctionBlock<'module> {
    pub fn new(function_builder: &'module FunctionBuilder<'module>, name: &str) -> Self {
        let name = CString::from_str(name).unwrap();
        // SAFETY: we know the function is a valid ref and name is a valid null-terminated C-string
        let block = unsafe { LLVMAppendBasicBlock(function_builder.as_llvm_ref(), name.as_ptr()) };

        Self {
            function_builder,
            block,
        }
    }

    pub fn build(&self, build: impl FnOnce(InstructionBuilder) -> TerminatorToken) {
        let instruction_builder = InstructionBuilder::new(self);

        build(instruction_builder);
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> *mut llvm_sys::LLVMBasicBlock {
        self.block
    }

    pub(crate) const fn function_builder(&self) -> &FunctionBuilder<'module> {
        self.function_builder
    }
}
