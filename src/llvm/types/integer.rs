use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMConstInt, LLVMInt64TypeInContext},
    prelude::{LLVMContextRef, LLVMTypeRef},
};

use super::{Type, value::Value};
use crate::llvm::{Context, LLVM_CONTEXT};

thread_local! {
    static U64_ID:IntegerType = IntegerType::new(LLVMInt64TypeInContext);
}

//TODO research if we should implement drop
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
    pub fn const_value(value: u64) -> Value<Self> {
        U64_ID.with(|r#type| unsafe { Value::new(LLVMConstInt(r#type.as_llvm_ref(), value, 0)) })
    }
}
