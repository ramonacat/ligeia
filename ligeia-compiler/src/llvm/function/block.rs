use std::{ffi::CString, marker::PhantomData, str::FromStr};

use llvm_sys::core::LLVMAppendBasicBlock;

use super::{
    FunctionBuilder,
    instruction_builder::{InstructionBuilder, TerminatorToken},
};

pub struct FunctionBlock<'function, 'module> {
    _phantom: PhantomData<&'function FunctionBuilder<'module>>,
    block: *mut llvm_sys::LLVMBasicBlock,
}

impl<'function, 'module> FunctionBlock<'function, 'module> {
    pub fn new(function: &'function FunctionBuilder<'module>, name: &str) -> Self {
        let name = CString::from_str(name).unwrap();
        // SAFETY: we know the function is a valid ref and name is a valid null-terminated C-string
        let block = unsafe { LLVMAppendBasicBlock(function.as_llvm_ref(), name.as_ptr()) };
        Self {
            _phantom: PhantomData,
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
}
