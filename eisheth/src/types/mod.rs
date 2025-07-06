pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

pub use array::Array;
pub use function::Function;
pub use integer::Integer;
use llvm_sys::prelude::LLVMTypeRef;
pub use pointer::Pointer;
pub use r#struct::Struct;

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
