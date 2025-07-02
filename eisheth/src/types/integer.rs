use std::marker::PhantomData;

use llvm_sys::{
    core::LLVMConstInt,
    prelude::{LLVMContextRef, LLVMTypeRef, LLVMValueRef},
};

use super::{Type, value::ConstValue};
use crate::{Context, LLVM_CONTEXT};

struct IntegerType {
    reference: LLVMTypeRef,
    zero_factory: Box<dyn Fn(&IntegerType) -> LLVMValueRef>,
    _phantom: PhantomData<&'static Context>,
}

impl Type for IntegerType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }

    fn const_uninitialized(&self) -> ConstValue {
        // SAFETY: zero_factory must always return a valid pointer
        unsafe { ConstValue::new((self.zero_factory)(self)) }
    }
}

impl IntegerType {
    pub fn new(
        factory: unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef,
        // This takes &IntegerType instead of LLVMTypeRef, so that we don't need to do weird dances
        // to make the closure unsafe
        zero_factory: impl Fn(&Self) -> LLVMValueRef + 'static,
    ) -> Self {
        Self {
            // SAFETY: The factory functions create types that only depend on the context, and we
            // keep a PhantomData reference to the context, so it won't be destroyed before the
            // types get dropped
            reference: LLVM_CONTEXT.with(|context| unsafe { factory(context.as_llvm_ref()) }),
            zero_factory: Box::new(zero_factory),
            _phantom: PhantomData,
        }
    }
}

macro_rules! declare_integer_type {
    ($bitcount:expr) => {
        paste::paste!{
            thread_local! {
                static [<U $bitcount _ID>]:IntegerType
                    = IntegerType::new(
                        llvm_sys::core::[<LLVMInt $bitcount TypeInContext>],
                        // SAFETY: We know the passed type is correct
                        |x| unsafe { LLVMConstInt(x.as_llvm_ref(), 0, 0) }

                    );
            }

            pub struct [<U $bitcount>];

            impl Type for [<U $bitcount>] {
                fn as_llvm_ref(&self) -> LLVMTypeRef {
                    [<U $bitcount _ID>].with(super::Type::as_llvm_ref)
                }

                fn const_uninitialized(&self) -> ConstValue {
                    [<U $bitcount _ID>].with(super::Type::const_uninitialized)
                }
            }

            impl [<U $bitcount>] {
                #[must_use]
                pub fn const_value(value: [<u $bitcount>]) -> ConstValue {
                    [<U $bitcount _ID>]
                        // SAFETY: the type held by `U64_ID` lives for 'static, so the reference for LLVMConstInt
                        // will be valid
                        .with(|r#type| unsafe {
                            ConstValue::new(
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
