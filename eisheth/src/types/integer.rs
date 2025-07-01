use std::marker::PhantomData;

use llvm_sys::prelude::{LLVMContextRef, LLVMTypeRef};

use super::{Type, value::Value};
use crate::{Context, LLVM_CONTEXT};

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

macro_rules! declare_integer_type {
    ($bitcount:expr) => {
        paste::paste!{
            thread_local! {
                static [<U $bitcount _ID>]:IntegerType
                    = IntegerType::new(llvm_sys::core::[<LLVMInt $bitcount TypeInContext>]);
            }

            pub struct [<U $bitcount>];

            impl Type for [<U $bitcount>] {
                fn as_llvm_ref(&self) -> LLVMTypeRef {
                    [<U $bitcount _ID>].with(super::Type::as_llvm_ref)
                }
            }

            impl [<U $bitcount>] {
                #[must_use]
                pub fn const_value(value: [<u $bitcount>]) -> Value {
                    [<U $bitcount _ID>]
                        // SAFETY: the type held by `U64_ID` lives for 'static, so the reference for LLVMConstInt
                        // will be valid
                        .with(|r#type| unsafe {
                            Value::new(
                                llvm_sys::core::LLVMConstInt(r#type.as_llvm_ref(), u64::from(value), 0)
                            )
                        })
                }
            }
        }
    };
}

declare_integer_type!(64);
declare_integer_type!(32);
