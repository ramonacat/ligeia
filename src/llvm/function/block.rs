use std::{ffi::CString, str::FromStr};

use llvm_sys::core::LLVMAppendBasicBlock;

use super::{FunctionBuilder, instruction_builder::InstructionBuilder};

// TODO should we implement drop for this at all?
pub struct FunctionBlock<'function, 'module> {
    #[allow(unused)] // TODO should this be PhantomData?
    function: &'function FunctionBuilder<'module>,
    block: *mut llvm_sys::LLVMBasicBlock,
}

impl<'function, 'module> FunctionBlock<'function, 'module> {
    pub fn new(function: &'function FunctionBuilder<'module>, name: &str) -> Self {
        let name = CString::from_str(name).unwrap();
        let block = unsafe { LLVMAppendBasicBlock(function.as_llvm_ref(), name.as_ptr()) };
        Self { function, block }
    }

    pub fn build(&self, build: impl FnOnce(InstructionBuilder)) {
        let instruction_builder = InstructionBuilder::new(self);

        build(instruction_builder);
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> *mut llvm_sys::LLVMBasicBlock {
        self.block
    }
}
