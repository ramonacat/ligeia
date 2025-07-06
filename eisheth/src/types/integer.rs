use std::marker::PhantomData;

use llvm_sys::{core::LLVMConstInt, prelude::LLVMTypeRef};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    types::RepresentedAs,
    value::ConstValue,
};

#[derive(Debug, Clone, Copy)]
pub struct IntegerType<const TBITS: usize> {
    reference: LLVMTypeRef,
    _phantom: PhantomData<&'static Context>,
}

impl<const TBITS: usize> Type for IntegerType<TBITS> {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

macro_rules! declare_integer_type {
    ($bitcount:expr) => {
        paste::paste!{
            impl IntegerType<$bitcount> {
                fn new() -> Self {
                    Self {
                        // SAFETY: We have a valid context
                        reference: LLVM_CONTEXT.with(|context| unsafe {
                            ::llvm_sys::core::[<LLVMInt $bitcount TypeInContext>](context.as_llvm_ref())
                        }),
                        _phantom: PhantomData,
                    }
                }

                #[must_use]
                pub fn const_value(&self, x: [<u $bitcount>]) -> crate::value::ConstValue {
                    // SAFETY: The reference to the type is valid
                    let value = unsafe { LLVMConstInt(self.reference, u64::from(x), 0) };

                    // SAFETY: The value just got created
                    unsafe { ConstValue::new(value) }
                }
            }

            thread_local! {
                static [<U $bitcount _ID>]:IntegerType<$bitcount>
                    = IntegerType::<$bitcount>::new();
            }

            impl RepresentedAs for [<u $bitcount>] {
                type RepresentationType = IntegerType<$bitcount>;

                fn representation() -> IntegerType<$bitcount> {
                    [<U $bitcount _ID>].with(|x| *x)
                }
            }

            impl Type for [<u $bitcount>] {
                fn as_llvm_ref(&self) -> LLVMTypeRef {
                    [<U $bitcount _ID>].with(super::Type::as_llvm_ref)
                }
            }
        }
    };
}

declare_integer_type!(64);
declare_integer_type!(32);
declare_integer_type!(16);
declare_integer_type!(8);
