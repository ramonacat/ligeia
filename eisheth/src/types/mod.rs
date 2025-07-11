pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

pub use array::Array;
pub use function::Function;
pub use integer::Integer;
use llvm_sys::{
    core::{LLVMConstBitCast, LLVMSizeOf},
    prelude::LLVMTypeRef,
};
pub use pointer::Pointer;
pub use r#struct::Struct;

use crate::{types::void::VoidType, value::ConstValue};

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

#[derive(Debug, Clone, Copy)]
pub enum TypeEnum {
    Void(VoidType),
    U8(Integer<u8>),
    U16(Integer<u16>),
    U32(Integer<u32>),
    U64(Integer<u64>),
    Pointer(Pointer),
}

impl Type for TypeEnum {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        match self {
            Self::Void(void_type) => void_type.as_llvm_ref(),
            Self::Pointer(pointer) => pointer.as_llvm_ref(),
            Self::U8(integer) => integer.as_llvm_ref(),
            Self::U16(integer) => integer.as_llvm_ref(),
            Self::U32(integer) => integer.as_llvm_ref(),
            Self::U64(integer) => integer.as_llvm_ref(),
        }
    }
}

impl From<Integer<u8>> for TypeEnum {
    fn from(value: Integer<u8>) -> Self {
        Self::U8(value)
    }
}

impl From<Integer<u16>> for TypeEnum {
    fn from(value: Integer<u16>) -> Self {
        Self::U16(value)
    }
}

impl From<Integer<u32>> for TypeEnum {
    fn from(value: Integer<u32>) -> Self {
        Self::U32(value)
    }
}

impl From<Integer<u64>> for TypeEnum {
    fn from(value: Integer<u64>) -> Self {
        Self::U64(value)
    }
}

impl From<Pointer> for TypeEnum {
    fn from(value: Pointer) -> Self {
        Self::Pointer(value)
    }
}
