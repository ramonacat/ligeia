use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMCountParamTypes, LLVMFunctionType, LLVMGetParamTypes},
    prelude::LLVMTypeRef,
};

use super::Type;
use crate::{Context, value::ConstValue};

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

    fn const_uninitialized(&self) -> Option<ConstValue> {
        None
    }
}

impl Function {
    /// # Panics
    /// If there are more params for the function than an u32 can hold. If this happens, you might
    /// want to consider refactoring your code.
    pub fn new(r#return: &dyn Type, arguments: &[&dyn Type]) -> Self {
        let mut param_types: Vec<_> = arguments.iter().map(|x| x.as_llvm_ref()).collect();

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

    pub(crate) fn get_argument(&self, index: usize) -> Option<LLVMTypeRef> {
        // SAFETY: We know that reference is valid till self is dropped
        let param_types_count = unsafe { LLVMCountParamTypes(self.reference) } as usize;

        if index >= param_types_count {
            return None;
        }

        let mut param_types: Vec<LLVMTypeRef> = Vec::with_capacity(param_types_count);

        // SAFETY: We know that param_types is alive at least for duration of the call, and
        // reference won't be freed until self is dropped
        unsafe { LLVMGetParamTypes(self.reference, param_types.as_mut_ptr()) };

        // SAFETY: LLVMGetParamTypes has filled the buffer, so now the pointers are correct for the
        // new len
        unsafe { param_types.set_len(param_types_count) };

        param_types.get(index).copied()
    }
}
