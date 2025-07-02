use llvm_sys::{
    core::{LLVMConstStructInContext, LLVMStructType},
    prelude::LLVMTypeRef,
};

use crate::{
    LLVM_CONTEXT,
    types::{
        self, Type,
        value::{ConstValue, Value},
    },
};

thread_local! {
    pub(super) static GLOBAL_INITIALIZERS_ENTRY_TYPE: InitializersEntryType = InitializersEntryType::new();
}

pub(super) struct InitializersEntryType(LLVMTypeRef);

impl InitializersEntryType {
    fn new() -> Self {
        let mut initializer_element_types = vec![
            types::U32.as_llvm_ref(),
            types::Pointer.as_llvm_ref(),
            types::Pointer.as_llvm_ref(),
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
}

impl Type for InitializersEntryType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.0
    }

    fn const_uninitialized(&self) -> types::value::ConstValue {
        let mut values = vec![
            types::U32.const_uninitialized().as_llvm_ref(),
            types::Pointer.const_uninitialized().as_llvm_ref(),
            types::Pointer.const_uninitialized().as_llvm_ref(),
        ];

        // SAFETY: we know the context is valid, and values are all matching the definition of the
        // type
        let result = LLVM_CONTEXT.with(|context| unsafe {
            LLVMConstStructInContext(
                context.as_llvm_ref(),
                values.as_mut_ptr(),
                u32::try_from(values.len()).unwrap(),
                0,
            )
        });

        // SAFETY: We just crated the result, it's a valid pointer to a value
        unsafe { ConstValue::new(result) }
    }
}
