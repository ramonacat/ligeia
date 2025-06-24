use super::module::built::Module;

mod block;
pub mod builder;
mod instruction_builder;

#[allow(unused)]
pub struct Function<'module> {
    module: &'module Module,
    function: *mut llvm_sys::LLVMValue,
    r#type: &'module super::types::function::FunctionType,
}

impl<'module> Function<'module> {
    pub(crate) fn new(
        module: &'module Module,
        function: *mut llvm_sys::LLVMValue,
        r#type: &'module super::types::function::FunctionType
    ) -> Self {
        Self { module, function, r#type }
    }
}
