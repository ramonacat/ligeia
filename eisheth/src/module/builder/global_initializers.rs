use llvm_sys::{
    core::{LLVMArrayType2, LLVMStructType},
    prelude::LLVMTypeRef,
};

use crate::types::{self, Type};

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

    // TODO We probably should have types::Array, this is hacky
    pub(super) fn array_ref(&self, len: usize) -> LLVMTypeRef {
        // SAFETY: The initializers_type was just crated, so it's valid
        unsafe { LLVMArrayType2(self.0, len as u64) }
    }
}

impl Type for InitializersEntryType {
    fn as_llvm_ref(&self) -> LLVMTypeRef {
        self.0
    }
}
