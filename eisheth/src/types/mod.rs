pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

pub use array::Array;
pub use function::Function;
pub use integer::Integer;
use llvm_sys::{core::LLVMSizeOf, prelude::LLVMTypeRef};
pub use pointer::Pointer;
pub use r#struct::Struct;

use crate::value::ConstValue;

pub trait RepresentedAs {
    type RepresentationType: Type;

    fn representation() -> Self::RepresentationType;
}

pub trait Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}

impl<T: RepresentedAs<RepresentationType = TRepresentation>, TRepresentation: Type> Type for T {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        T::representation().as_llvm_ref()
    }
}

pub trait TypeExtensions {
    fn sizeof(&self) -> ConstValue;
}

impl<T: Type + ?Sized> TypeExtensions for T {
    fn sizeof(&self) -> ConstValue {
        // SAFETY: The type reference comes from a valid wrapper
        let result = unsafe { LLVMSizeOf(self.as_llvm_ref()) };

        // SAFETY: We just created the result, it is a valid pointer
        unsafe { ConstValue::new(result) }
    }
}
