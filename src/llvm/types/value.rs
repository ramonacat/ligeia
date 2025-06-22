use std::marker::PhantomData;

use llvm_sys::prelude::LLVMValueRef;

use super::Type;

pub struct Value<T: Type> {
    reference: LLVMValueRef,
    _phantom: PhantomData<Box<T>>,
}
impl<T: Type> Value<T> {
    pub(in crate::llvm) const unsafe fn new(value: *mut llvm_sys::LLVMValue) -> Self {
        Self {
            reference: value,
            _phantom: PhantomData,
        }
    }

    pub(crate) const fn as_llvm_ref(&self) -> *mut llvm_sys::LLVMValue {
        self.reference
    }
}
