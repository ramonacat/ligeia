use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

mod block;
pub mod builder;
mod instruction_builder;

pub struct Function {
    // TODO use it!
    #[allow(unused)]
    value: LLVMValueRef,
    // TODO use it!
    #[allow(unused)]
    r#type: LLVMTypeRef,
}

impl Function {
    pub const unsafe fn new(value: LLVMValueRef, r#type: LLVMTypeRef) -> Self {
        Self { value, r#type }
    }
}
