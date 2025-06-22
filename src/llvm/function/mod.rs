mod block;
mod instruction_builder;

use std::{ffi::CString, marker::PhantomData, str::FromStr};

use block::FunctionBlock;
use llvm_sys::{
    core::{LLVMAddFunction, LLVMGetParam},
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
        let _argument_type = self.r#type.get_argument(index as usize)?;

        // TODO figure out what's wrong with this assert
        // assert!(argument_type.type_id() == TypeId::of::<TType>());

        let argument = unsafe { LLVMGetParam(self.function, index) };

        Some(unsafe { Value::new(argument) })
    }
}
