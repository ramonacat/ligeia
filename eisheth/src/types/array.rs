use llvm_sys::{
    core::{LLVMArrayType2, LLVMConstArray2},
    prelude::LLVMTypeRef,
};

use crate::{
    types::Type,
    value::{ConstValue, Value},
};

#[derive(Debug, Clone, Copy)]
pub struct Array<T: Type> {
    reference: LLVMTypeRef,
    element_type: T,
}

impl<T: Type> Array<T> {
    pub fn new(element_type: T, len: usize) -> Self {
        // SAFETY: We know the element_type is a valid type
        let reference = unsafe { LLVMArrayType2(element_type.as_llvm_ref(), len as u64) };
        Self {
            reference,
            element_type,
        }
    }

    pub(crate) fn const_values(&self, initializer_values: &[ConstValue]) -> ConstValue {
        let mut values: Vec<_> = initializer_values.iter().map(Value::as_llvm_ref).collect();

        // SAFETY: The values are of correct type and valid pointers, the length matches, and
        // element_type is a vaid pointer
        let result = unsafe {
            LLVMConstArray2(
                self.element_type.as_llvm_ref(),
                values.as_mut_ptr(),
                u64::try_from(values.len()).unwrap(),
            )
        };

        // SAFETY: We just created the result, it is valid
        unsafe { ConstValue::new(result) }
    }
}

impl<T: Type> Type for Array<T> {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}
