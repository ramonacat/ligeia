use std::marker::PhantomData;

use llvm_sys::{core::LLVMConstInt, prelude::LLVMTypeRef};

use super::Type;
use crate::{
    context::{Context, LLVM_CONTEXT},
    types::RepresentedAs,
    value::ConstValue,
};

#[derive(Debug, Clone, Copy)]
pub struct Integer<T> {
    reference: LLVMTypeRef,
    _type: PhantomData<T>,
    _context: PhantomData<&'static Context>,
}

impl<T: Copy> Type for Integer<T> {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.reference
    }
}

macro_rules! declare_integer_type {
    ($bitcount:expr) => {
        paste::paste!{
            impl Integer<[<u $bitcount>]> {
                fn new() -> Self {
                    Self {
                        // SAFETY: We have a valid context
                        reference: LLVM_CONTEXT.with(|context| unsafe {
                            ::llvm_sys::core::[<LLVMInt $bitcount TypeInContext>](context.as_llvm_ref())
                        }),
                        _type: PhantomData,
                        _context: PhantomData,
                    }
                }

                pub fn const_value(&self, x: [<u $bitcount>]) -> crate::value::ConstValue {
                    // SAFETY: The reference to the type is valid
                    let value = unsafe { LLVMConstInt(self.reference, u64::from(x), 0) };

                    // SAFETY: The value just got created
                    unsafe { ConstValue::new(value) }
                }
            }

            thread_local! {
                static [<U $bitcount _ID>]:Integer<[<u $bitcount>]>
                    = Integer::<[<u $bitcount>]>::new();
            }

            impl RepresentedAs for [<u $bitcount>] {
                type RepresentationType = Integer<[<u $bitcount>]>;

                fn representation() -> Integer<[<u $bitcount>]> {
                    [<U $bitcount _ID>].with(|x| *x)
                }
            }

            impl From<[<u $bitcount>]> for ConstValue {
                fn from(value: [<u $bitcount>]) -> Self {
                    [<U $bitcount _ID>].with(|x| x.const_value(value))
                }
            }
        }
    };
}

declare_integer_type!(64);
declare_integer_type!(32);
declare_integer_type!(16);
declare_integer_type!(8);
