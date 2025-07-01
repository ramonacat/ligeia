use llvm_sys::prelude::LLVMValueRef;

use super::{module::built::Module, types};

mod block;
pub mod builder;
pub mod declaration;
pub mod instruction_builder;

#[allow(unused)]
pub struct Function<'module> {
    module: &'module Module,
    reference: LLVMValueRef,
    r#type: types::Function,
}

impl<'module> Function<'module> {
    pub(crate) const fn new(
        module: &'module Module,
        reference: LLVMValueRef,
        r#type: types::Function,
    ) -> Self {
        Self {
            module,
            reference,
            r#type,
        }
    }
}
