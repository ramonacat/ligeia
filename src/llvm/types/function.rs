use llvm_sys::{core::LLVMFunctionType, prelude::LLVMTypeRef};

use super::Type;

// TODO research if we should implement drop and dispose of the LLVMTypeRef
pub struct FunctionType {
    reference: LLVMTypeRef,
    arguments: Box<[Box<dyn Type>]>,
}

impl Type for FunctionType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

impl FunctionType {
    pub fn new(r#return: &impl Type, arguments: Box<[Box<dyn Type>]>) -> Self {
        let mut param_types: Vec<LLVMTypeRef> = arguments.iter().map(|x| x.as_llvm_ref()).collect();

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
            arguments,
        }
    }

    pub(in crate::llvm) fn get_argument(&self, index: usize) -> Option<&(dyn Type + 'static)> {
        self.arguments.get(index).map(|x| &**x)
    }
}
