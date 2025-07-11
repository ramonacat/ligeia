use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMCountParamTypes, LLVMFunctionType},
    prelude::LLVMTypeRef,
};

use super::Type;
use crate::{context::Context, types::OpaqueType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Function {
    reference: LLVMTypeRef,
    // While the Function itself does not depend on the context, its constituent types (return,
    // arguments) do.
    _context: PhantomData<&'static Context>,
}

impl Type for Function {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

impl Function {
    /// # Panics
    /// If there are more params for the function than an u32 can hold. If this happens, you might
    /// want to consider refactoring your code.
    pub fn new<TReturn: Type>(r#return: TReturn, arguments: &[OpaqueType]) -> Self {
        let mut param_types: Vec<_> = arguments.iter().map(OpaqueType::as_llvm_ref).collect();

        Self {
            // SAFETY: This constructor needs it parameters alive only while it's being executed,
            // and we can guarantee that both `r#return` and `param_types` won't be dropped until
            // end of scope
            reference: unsafe {
                LLVMFunctionType(
                    r#return.as_llvm_ref(),
                    param_types.as_mut_ptr(),
                    u32::try_from(param_types.len()).unwrap(),
                    0,
                )
            },
            _context: PhantomData,
        }
    }

    pub(crate) fn arguments_count(&self) -> usize {
        // SAFETY: We know that reference is valid till self is dropped
        (unsafe { LLVMCountParamTypes(self.reference) }) as usize
    }
}
