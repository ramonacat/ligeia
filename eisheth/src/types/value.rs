use llvm_sys::prelude::LLVMValueRef;

// TODO: Separate type for a constant value?
// TODO: Should Value be generic over Type after all? we could then do more type-level checks
// (target for store is always a pointer, etc.).
pub struct Value {
    reference: LLVMValueRef,
}

impl Value {
    pub(crate) const unsafe fn new(value: LLVMValueRef) -> Self {
        Self { reference: value }
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMValueRef {
        self.reference
    }
}
