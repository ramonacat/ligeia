use llvm_sys::prelude::LLVMValueRef;

use super::module::built::Module;

mod block;
pub mod builder;
mod instruction_builder;

#[allow(unused)]
pub struct Function<'module> {
    module: &'module Module,
    reference: LLVMValueRef,
    r#type: super::types::function::Function,
}

impl<'module> Function<'module> {
    pub(crate) const fn new(
        module: &'module Module,
        reference: LLVMValueRef,
        r#type: super::types::function::Function,
    ) -> Self {
        Self {
            module,
            reference,
            r#type,
        }
    }
}
