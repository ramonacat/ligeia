pub mod function;
pub mod integer;
pub mod value;

use llvm_sys::prelude::LLVMTypeRef;

pub use function::Function;
pub use integer::U64;

pub trait Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}
