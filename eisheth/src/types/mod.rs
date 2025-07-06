pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

pub use array::Array;
pub use function::Function;
pub use integer::IntegerType;
use llvm_sys::prelude::LLVMTypeRef;
pub use pointer::PointerType;
pub use r#struct::Struct;
pub use void::Void;

pub trait RepresentedAs {
    type RepresentationType: Type;

    fn representation() -> Self::RepresentationType;
}

pub trait Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}
