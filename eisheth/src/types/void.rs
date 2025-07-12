use std::marker::PhantomData;

use llvm_sys::{core::LLVMVoidTypeInContext, prelude::LLVMTypeRef};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    types::RepresentedAs,
};

#[derive(Debug, Clone, Copy)]
pub struct Void {
    reference: LLVMTypeRef,
    _context: PhantomData<&'static Context>,
}

impl Type for Void {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

impl Void {
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
    static VOID_TYPE:Void = Void::new();
}

impl RepresentedAs for () {
    type RepresentationType = Void;

    fn representation() -> Self::RepresentationType {
        VOID_TYPE.with(|void| *void)
    }
}
