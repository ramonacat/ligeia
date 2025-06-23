mod block;
mod instruction_builder;

use std::{ffi::CString, marker::PhantomData, str::FromStr};

use block::FunctionBlock;
use llvm_sys::{
    core::{LLVMAddFunction, LLVMGetParam, LLVMTypeOf},
    prelude::LLVMValueRef,
};

use super::{
    module::Module,
    types::{self, Type, value::Value},
};

pub struct FunctionBuilder<'module> {
    function: LLVMValueRef,
    r#type: types::function::FunctionType,
    _phantom: PhantomData<&'module Module>,
}

impl<'module> FunctionBuilder<'module> {
    pub fn new(module: &'module Module, name: &str, r#type: types::function::FunctionType) -> Self {
        let name = CString::from_str(name).unwrap();

        let function =
        // SAFETY: The module is a valid module, the name is a null terminated string, and the type
        // exists for the duration of the call, so we're safe
            unsafe { LLVMAddFunction(module.as_llvm_ref(), name.as_ptr(), r#type.as_llvm_ref()) };

        Self {
            function,
            r#type,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn create_block<'function>(
        &'function self,
        name: &str,
    ) -> FunctionBlock<'function, 'module> {
        FunctionBlock::new(self, name)
    }

    const fn as_llvm_ref(&self) -> *mut llvm_sys::LLVMValue {
        self.function
    }

    pub(crate) fn get_argument<TType: Type + 'static>(&self, index: u32) -> Option<Value<TType>> {
        let argument_type = self.r#type.get_argument(index as usize)?;

        // SAFETY: We've ensured that the `index` is not out-of-bounds while getting the argument
        // type, and self.function is a valid reference to the function
        let argument = unsafe { LLVMGetParam(self.function, index) };

        // SAFETY: We've ensured LLVMGetParam got correct arguments, so `argument` must be valid
        assert!(argument_type.as_llvm_ref() == unsafe { LLVMTypeOf(argument) });

        // SAFETY: We know that the type of the argument matches the type of the Value, so this is
        // correct and safe
        // TODO: Should this constructor even be unsafe? It can create an incorrect Value, but
        // memory-safety-wise should be fine
        Some(unsafe { Value::new(argument) })
    }
}
