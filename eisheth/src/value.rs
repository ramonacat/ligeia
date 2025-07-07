use llvm_sys::{core::LLVMIsConstant, prelude::LLVMValueRef};

pub trait Value: Copy {
    fn as_llvm_ref(&self) -> LLVMValueRef;
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum ConstOrDynamicValue {
    Const(ConstValue),
    Dynamic(DynamicValue),
}

impl ConstOrDynamicValue {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        // SAFETY: The caller must've provided a valid `value` pointer
        if unsafe { LLVMIsConstant(value) } == 1 {
            // SAFETY: The caller must've provided a valid `value` pointer
            Self::Const(unsafe { ConstValue::new(value) })
        } else {
            // SAFETY: The caller must've provided a valid `value` pointer
            Self::Dynamic(unsafe { DynamicValue::new(value) })
        }
    }
}

impl Value for ConstOrDynamicValue {
    fn as_llvm_ref(&self) -> LLVMValueRef {
        match self {
            Self::Const(const_value) => const_value.as_llvm_ref(),
            Self::Dynamic(dynamic_value) => dynamic_value.as_llvm_ref(),
        }
    }
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct ConstValue {
    reference: LLVMValueRef,
}

impl ConstValue {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        // SAFETY: The caller must have ensured that the LLVMValueRef is valid
        assert!(unsafe { LLVMIsConstant(value) } == 1);
        Self { reference: value }
    }
}

impl Value for ConstValue {
    fn as_llvm_ref(&self) -> LLVMValueRef {
        self.reference
    }
}

impl From<ConstValue> for ConstOrDynamicValue {
    fn from(value: ConstValue) -> Self {
        Self::Const(value)
    }
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct DynamicValue {
    reference: LLVMValueRef,
}

impl DynamicValue {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        // SAFETY: The caller must have ensured that the LLVMValueRef is valid
        assert!(unsafe { LLVMIsConstant(value) } == 0);
        Self { reference: value }
    }
}

impl Value for DynamicValue {
    fn as_llvm_ref(&self) -> LLVMValueRef {
        self.reference
    }
}

impl From<DynamicValue> for ConstOrDynamicValue {
    fn from(value: DynamicValue) -> Self {
        Self::Dynamic(value)
    }
}
