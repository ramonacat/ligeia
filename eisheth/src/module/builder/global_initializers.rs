use llvm_sys::{
    core::{LLVMConstStructInContext, LLVMStructType},
    prelude::LLVMTypeRef,
};

use crate::{
    context::LLVM_CONTEXT,
    types::{self, RepresentedAs, Type},
    value::{ConstValue, Value},
};

thread_local! {
    pub(super) static GLOBAL_INITIALIZERS_ENTRY_TYPE: InitializersEntryType = InitializersEntryType::new();
}

pub(super) struct InitializersEntryType(LLVMTypeRef);

impl InitializersEntryType {
    fn new() -> Self {
        let mut initializer_element_types = vec![
            u32::representation().as_llvm_ref(),
            <*mut fn()>::representation().as_llvm_ref(),
            <*mut u8>::representation().as_llvm_ref(),
        ];

        // SAFETY: The element types are valid for the duration of the call
        let initializers_type = unsafe {
            LLVMStructType(
                initializer_element_types.as_mut_ptr(),
                u32::try_from(initializer_element_types.len()).unwrap(),
                0,
            )
        };

        Self(initializers_type)
    }

    pub(crate) fn const_values(
        priority: &ConstValue,
        initializer: &ConstValue,
        initialized_value: Option<&ConstValue>,
    ) -> ConstValue {
        let mut values = vec![
            priority.as_llvm_ref(),
            initializer.as_llvm_ref(),
            initialized_value.map_or_else(
                || types::PointerType::const_null().as_llvm_ref(),
                Value::as_llvm_ref,
            ),
        ];

        // SAFETY: The values were crated from valid wrapper objects, the pointer and length are
        // valid, the context is valid
        let result = LLVM_CONTEXT.with(|context| unsafe {
            LLVMConstStructInContext(
                context.as_llvm_ref(),
                values.as_mut_ptr(),
                u32::try_from(values.len()).unwrap(),
                0,
            )
        });

        // SAFETY: We just created the value, it is correct
        unsafe { ConstValue::new(result) }
    }
}

impl Type for InitializersEntryType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.0
    }
}
