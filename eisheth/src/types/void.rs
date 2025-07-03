use std::marker::PhantomData;

use llvm_sys::{core::LLVMVoidTypeInContext, prelude::LLVMTypeRef};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    value::ConstValue,
};

pub struct VoidType {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl Type for VoidType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }

    fn const_uninitialized(&self) -> Option<ConstValue> {
        None
    }
}

impl VoidType {
    fn new() -> Self {
        let reference =
            // SAFETY: We know the context is &'static so this is safe
            LLVM_CONTEXT.with(|context| unsafe { LLVMVoidTypeInContext(context.as_llvm_ref()) });
        Self {
            reference,
            _context: PhantomData,
        }
    }
}

thread_local! {
    static VOID_TYPE:VoidType = VoidType::new();
}

pub struct Void;

impl Type for Void {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        VOID_TYPE.with(Type::as_llvm_ref)
    }

    fn const_uninitialized(&self) -> Option<ConstValue> {
        None
    }
}
