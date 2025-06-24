use std::{ffi::CString, str::FromStr as _};

use llvm_sys::{
    core::{LLVMAddFunction, LLVMGetParam, LLVMTypeOf},
    prelude::LLVMValueRef,
};

use super::block::FunctionBlock;
use crate::llvm::{
    module::Module,
    types::{self, Type, value::Value},
};

pub struct FunctionBuilder<'symbols, 'module> {
    function: LLVMValueRef,
    r#type: types::function::FunctionType,
    module: &'module Module<'symbols>,
}

impl<'symbols, 'module> FunctionBuilder<'symbols, 'module> {
    pub fn new(
        module: &'module Module<'symbols>,
        name: &str,
        r#type: types::function::FunctionType,
    ) -> Self {
        let name = CString::from_str(name).unwrap();

        let function =
        // SAFETY: The module is a valid module, the name is a null terminated string, and the type
        // exists for the duration of the call, so we're safe
            unsafe { LLVMAddFunction(module.as_llvm_ref(), name.as_ptr(), r#type.as_llvm_ref()) };

        Self {
            function,
            r#type,
            module,
        }
    }

    pub(crate) fn create_block<'function>(
        &'function self,
        name: &str,
    ) -> FunctionBlock<'symbols, 'function, 'module> {
        FunctionBlock::new(self, name)
    }

    pub(in crate::llvm) const fn as_llvm_ref(&self) -> *mut llvm_sys::LLVMValue {
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
        Some(unsafe { Value::new(argument) })
    }

    pub(in crate::llvm) const fn build(&self) -> LLVMValueRef {
        self.function
    }

    pub(crate) const fn module(&self) -> &Module<'symbols> {
        self.module
    }
}
