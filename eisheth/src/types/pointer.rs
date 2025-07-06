use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMConstIntToPtr, LLVMConstPointerNull, LLVMPointerTypeInContext},
    prelude::LLVMTypeRef,
};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    types::RepresentedAs,
    value::{ConstValue, Value},
};

#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl Pointer {
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

    pub fn const_null() -> ConstValue {
        // SAFETY: the pointer type is always valid
        let result = POINTER.with(|x| unsafe { LLVMConstPointerNull(x.as_llvm_ref()) });

        // SAFETY: We just crated the result, it is valid
        unsafe { ConstValue::new(result) }
    }
}

impl Type for Pointer {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

thread_local! {
    static POINTER:Pointer = Pointer::new();
}

impl<T> RepresentedAs for *mut T {
    type RepresentationType = Pointer;

    fn representation() -> Self::RepresentationType {
        POINTER.with(|x| *x)
    }
}

impl<T> Type for *mut T {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        <Self as RepresentedAs>::representation().as_llvm_ref()
    }
}

impl<T> From<*mut T> for ConstValue {
    fn from(value: *mut T) -> Self {
        // SAFETY: the type pointer is valid
        let result = POINTER.with(|r#type| unsafe {
            LLVMConstIntToPtr(
                u64::representation()
                    .const_value(value as u64)
                    .as_llvm_ref(),
                r#type.as_llvm_ref(),
            )
        });

        // SAFETY: We just crated the value and it's valid
        unsafe { Self::new(result) }
    }
}

impl<T> RepresentedAs for *const T {
    type RepresentationType = Pointer;

    fn representation() -> Self::RepresentationType {
        POINTER.with(|x| *x)
    }
}

impl<T> Type for *const T {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        <Self as RepresentedAs>::representation().as_llvm_ref()
    }
}

impl<T> From<*const T> for ConstValue {
    fn from(value: *const T) -> Self {
        // SAFETY: the type pointer is valid
        let result = POINTER.with(|r#type| unsafe {
            LLVMConstIntToPtr(
                u64::representation()
                    .const_value(value as u64)
                    .as_llvm_ref(),
                r#type.as_llvm_ref(),
            )
        });

        // SAFETY: We just crated the value and it's valid
        unsafe { Self::new(result) }
    }
}
