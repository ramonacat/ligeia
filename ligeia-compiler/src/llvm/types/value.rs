use llvm_sys::prelude::LLVMValueRef;

pub struct Value {
    reference: LLVMValueRef,
}

impl Value {
    pub(in crate::llvm) const unsafe fn new(value: LLVMValueRef) -> Self {
        Self { reference: value }
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> LLVMValueRef {
        self.reference
    }
}
