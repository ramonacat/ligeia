use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMConstNull, LLVMPointerTypeInContext},
    prelude::LLVMTypeRef,
};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    value::ConstValue,
};

struct PointerType {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl PointerType {
    fn new() -> Self {
        let reference = LLVM_CONTEXT.with(|context| {
            // SAFETY: We know the context is valid, therefore the preconditions for this call are
            // satisfied
            unsafe { LLVMPointerTypeInContext(context.as_llvm_ref(), 0) }
        });

        Self {
            reference,
            _context: PhantomData,
        }
    }
}

impl Type for PointerType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

thread_local! {
    static POINTER:PointerType = PointerType::new();
}

// TODO can we instead impelement RepresentedAs for *mut T and *const T?
pub struct Pointer;

impl Pointer {
    #[must_use]
    pub fn const_null() -> ConstValue {
        // SAFETY: The type reference is definitely a valid pointer
        let result = POINTER.with(|r#type| unsafe { LLVMConstNull(r#type.as_llvm_ref()) });

        // SAFETY: We just created the value
        unsafe { ConstValue::new(result) }
    }
}

impl Type for Pointer {
    fn as_llvm_ref(&self) -> llvm_sys::prelude::LLVMTypeRef {
        POINTER.with(super::Type::as_llvm_ref)
    }
}
