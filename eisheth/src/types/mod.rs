pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

use std::marker::PhantomData;

pub use array::Array;
pub use function::Function;
pub use integer::Integer;
use llvm_sys::{
    core::{LLVMConstBitCast, LLVMSizeOf},
    prelude::LLVMTypeRef,
};
pub use pointer::Pointer;
pub use r#struct::Struct;

use crate::value::ConstValue;

pub trait RepresentedAs {
    type RepresentationType: Type;

    fn representation() -> Self::RepresentationType;
}

pub trait Type: Copy {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}

impl<T: RepresentedAs<RepresentationType = TRepresentation> + Copy, TRepresentation: Type> Type
    for T
{
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        T::representation().as_llvm_ref()
    }
}

pub trait TypeExtensions {
    fn sizeof(&self) -> ConstValue;
}

impl<T: Type> TypeExtensions for T {
    fn sizeof(&self) -> ConstValue {
        // SAFETY: The type reference comes from a valid wrapper
        let result = unsafe { LLVMSizeOf(self.as_llvm_ref()) };

        // SAFETY: The type reference is from a safe wrapper, so it's valid
        let result = unsafe { LLVMConstBitCast(result, u64::representation().as_llvm_ref()) };

        // SAFETY: We just created the result, it is a valid pointer
        unsafe { ConstValue::new(result) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpaqueType(LLVMTypeRef, PhantomData<&'static crate::context::Context>);

impl OpaqueType {
    pub(crate) const unsafe fn new(reference: LLVMTypeRef) -> Self {
        Self(reference, PhantomData)
    }

    pub(crate) const fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.0
    }
}

impl<T: Type> From<T> for OpaqueType {
    fn from(value: T) -> Self {
        // SAFETY: we take the reference from a valid object, and types are never destroyed
        unsafe { Self::new(value.as_llvm_ref()) }
    }
}
