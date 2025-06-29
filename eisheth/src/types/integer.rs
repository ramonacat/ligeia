use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMConstInt, LLVMInt64TypeInContext},
    prelude::{LLVMContextRef, LLVMTypeRef},
};

use super::{Type, value::Value};
use crate::{Context, LLVM_CONTEXT};

thread_local! {
    static U64_ID:IntegerType = IntegerType::new(LLVMInt64TypeInContext);
}

#[derive(Clone, Copy)]
struct IntegerType {
    reference: LLVMTypeRef,
    _phantom: PhantomData<&'static Context>,
}

impl Type for IntegerType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

impl IntegerType {
    pub fn new(factory: unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef) -> Self {
        Self {
            // SAFETY: The factory functions create types that only depend on the context, and we
            // keep a PhantomData reference to the context, so it won't be destroyed before the
            // types get dropped
            reference: LLVM_CONTEXT.with(|context| unsafe { factory(context.as_llvm_ref()) }),
            _phantom: PhantomData,
        }
    }
}

pub struct U64;

impl Type for U64 {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        U64_ID.with(super::Type::as_llvm_ref)
    }
}

impl U64 {
    #[must_use]
    pub fn const_value(value: u64) -> Value {
        // SAFETY: the type held by `U64_ID` lives for 'static, so the reference for LLVMConstInt
        // will be valid
        U64_ID.with(|r#type| unsafe { Value::new(LLVMConstInt(r#type.as_llvm_ref(), value, 0)) })
    }
}
