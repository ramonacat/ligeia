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

    // TODO should this be moved to a separate trait? same as the one with from_value or yet
    // another one? Or maybe we could just avoid having this method at all? This is mostly for
    // globals, and there should probably be a way to tell LLVM to allocate and just not initialize
    // to anything in particular?
    fn const_uninitialized(&self) -> Option<ConstValue>;
}

// TODO this is for cases like vector, where we need to know the type without necessairly having an
// instance
pub trait StaticType: Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}

// TODO Add a subtrait that has `fn from_value(value: T) -> ConstValue`
