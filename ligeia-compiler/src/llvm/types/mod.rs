pub mod function;
pub mod integer;
pub mod value;

pub use function::Function;
pub use integer::U64;
use llvm_sys::prelude::LLVMTypeRef;

pub trait Type {
    fn as_llvm_ref(&self) -> LLVMTypeRef;
}
