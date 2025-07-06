pub mod array;
pub mod function;
pub mod integer;
pub mod pointer;
pub mod r#struct;
pub mod void;

pub use array::Array;
pub use function::Function;
pub use integer::{U8, U16, U32, U64};
use llvm_sys::prelude::LLVMTypeRef;
pub use pointer::Pointer;
pub use r#struct::Struct;
pub use void::Void;

use crate::value::ConstValue;

pub trait RepresentedAs {
    type RepresentationType: Type;
    const REPRESENTATION: Self::RepresentationType;
}

pub trait Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}

// TODO Add a subtrait that has `fn from_value(value: T) -> ConstValue`
