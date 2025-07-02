use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMConstPointerNull, LLVMPointerTypeInContext},
    prelude::LLVMTypeRef,
};

use super::Type;
use crate::{Context, LLVM_CONTEXT, types::value::ConstValue};

struct PointerType {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl PointerType {
    fn new() -> Self {
        let reference = LLVM_CONTEXT.with(|context| {
            // TODO Support more than a single address space
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

    fn const_uninitialized(&self) -> Option<ConstValue> {
        // SAFETY: The reference to the type is valid, so it's all chill.
        Some(unsafe { ConstValue::new(LLVMConstPointerNull(self.reference)) })
    }
}

thread_local! {
    static POINTER:PointerType = PointerType::new();
}

// TODO should pointers have optional type that can be verified in instruction builder, or do we
// leave it for the generated LLVM IR check?
pub struct Pointer;

impl Pointer {
    #[must_use]
    // TODO should this still exist if const_uninitialized is a thing?
    pub fn const_null() -> ConstValue {
        // SAFETY: We know the pointer type pointer is valid
        let value = POINTER.with(|pointer| unsafe { LLVMConstPointerNull(pointer.as_llvm_ref()) });

        // SAFETY: We just crated the value, it is valid
        unsafe { ConstValue::new(value) }
    }
}

impl Type for Pointer {
    fn as_llvm_ref(&self) -> llvm_sys::prelude::LLVMTypeRef {
        POINTER.with(super::Type::as_llvm_ref)
    }

    fn const_uninitialized(&self) -> Option<ConstValue> {
        POINTER.with(super::Type::const_uninitialized)
    }
}
